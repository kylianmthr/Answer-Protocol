use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::broadcast::broadcast_room;
use crate::logs_format::log_output;
use crate::state::{SharedState, Turn};

const PLAYER_MIN: i32 = 20;
const PLAYER_MAX: i32 = 30;
const NPC_MIN: i32 = 20;
const NPC_MAX: i32 = 30;
pub const MAX_HP: i32 = 100;
pub const RESPAWN_HP: i32 = 50;
const DEFEND_DAMAGE_PERCENT: i32 = 50;
const RIPOSTE_PERCENT: i32 = 50;
const FLEE_CHANCE_PERCENT: i32 = 50;

fn roll(min: i32, max: i32, salt: u64) -> i32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    let mut x = nanos ^ salt.wrapping_mul(0x9E3779B97F4A7C15);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    let span = (max - min + 1).max(1) as u64;
    min + (x % span) as i32
}

fn combat_json(
    actor: &str,
    attacker_hp: i32,
    target_hp: i32,
    damage: i32,
    status: &str,
    enemy: &str,
    action: &str,
) -> String {
    serde_json::json!({
        "actor": actor,
        "attacker_hp": attacker_hp,
        "target_hp": target_hp,
        "damage": damage,
        "status": status,
        "enemy": enemy,
        "action": action,
    })
    .to_string()
}

async fn resolve_npc_in_room(
    npc_name_or_id: &str,
    room: &str,
    state: &Arc<SharedState>,
) -> Option<(String, bool, i32)> {
    let world_data = state.world_data.lock().await;
    world_data
        .world
        .npcs
        .iter()
        .find(|(id, npc)| {
            (id.as_str() == npc_name_or_id || npc.name == npc_name_or_id) && npc.room == room
        })
        .map(|(id, npc)| (id.clone(), npc.hostile, npc.hp))
}

async fn respawn_player(
    username: &str,
    npc_id: &str,
    npc_max_hp: i32,
    state: Arc<SharedState>,
) -> (String, i32) {
    let initial_room = {
        let world_data = state.world_data.lock().await;
        world_data.world.initial_room.clone()
    };
    {
        let mut world_state = state.world_state.lock().await;
        if let Some(npc) = world_state.npcs.get_mut(npc_id) {
            npc.hp = npc_max_hp;
        }
    }
    let old_room = {
        let mut players = state.players.lock().await;
        let player = players.get_mut(username).unwrap();
        let old = player.room.clone();
        player.hp = RESPAWN_HP;
        player.room = initial_room.clone();
        player.combat_turn = Turn::Player;
        player.combat_target = None;
        old
    };
    {
        let mut world_state = state.world_state.lock().await;
        if let Some(room) = world_state.room.get_mut(&old_room) {
            room.players.retain(|p| p != username);
        }
        if let Some(room) = world_state.room.get_mut(&initial_room) {
            room.players.push(username.to_string());
        }
    }
    log_output("WARN", "COMBAT_RESULT", serde_json::json!({
        "player": username, "npc": npc_id, "result": "player_defeated", "respawn_hp": RESPAWN_HP
    }));
    broadcast_room(
        &old_room,
        &format!("EVT ROOM COMBAT {} was defeated by {}", username, npc_id),
        Arc::clone(&state),
    )
    .await;
    (initial_room, RESPAWN_HP)
}

pub async fn attack(username: String, npc_name_or_id: &str, state: Arc<SharedState>) -> String {
    let player_room = {
        let players = state.players.lock().await;
        match players.get(&username) {
            Some(player) => player.room.clone(),
            None => return "ERR 404 NPC_NOT_FOUND".to_string(),
        }
    };
    let (npc_id, hostile, npc_max_hp) =
        match resolve_npc_in_room(npc_name_or_id, &player_room, &state).await {
            Some(v) => v,
            None => return "ERR 404 NPC_NOT_FOUND".to_string(),
        };

    if !hostile {
        return "ERR 405 NPC_NOT_HOSTILE".to_string();
    }

    let turn = {
        let mut players = state.players.lock().await;
        let player = players.get_mut(&username).unwrap();
        player.combat_target = Some(npc_id.clone());
        player.combat_turn.clone()
    };

    if turn == Turn::Player {
        let player_dmg = roll(PLAYER_MIN, PLAYER_MAX, 1);
        let target_hp = {
            let mut world_state = state.world_state.lock().await;
            let npc = world_state.npcs.get_mut(&npc_id).unwrap();
            npc.hp = (npc.hp - player_dmg).max(0);
            npc.hp
        };
        let attacker_hp = {
            let players = state.players.lock().await;
            players.get(&username).map(|p| p.hp).unwrap_or(0)
        };

        if target_hp <= 0 {
            {
                let mut players = state.players.lock().await;
                let player = players.get_mut(&username).unwrap();
                player.combat_turn = Turn::Player;
                player.combat_target = None;
            }
            log_output("INFO", "COMBAT_RESULT", serde_json::json!({
                "player": username, "npc": npc_id, "action": "attack",
                "damage": player_dmg, "result": "victory"
            }));
            broadcast_room(
                &player_room,
                &format!("EVT ROOM COMBAT {} defeated {}", username, npc_id),
                Arc::clone(&state),
            )
            .await;
            return format!(
                "OK {}",
                combat_json(
                    "player",
                    attacker_hp,
                    0,
                    player_dmg,
                    "victory",
                    &npc_id,
                    "attack"
                )
            );
        }
        {
            let mut players = state.players.lock().await;
            players.get_mut(&username).unwrap().combat_turn = Turn::Enemy;
        }
        log_output("INFO", "COMBAT_RESULT", serde_json::json!({
            "player": username, "npc": npc_id, "action": "attack",
            "damage": player_dmg, "target_hp": target_hp, "result": "hit"
        }));
        return format!(
            "OK {}",
            combat_json(
                "player",
                attacker_hp,
                target_hp,
                player_dmg,
                "combat",
                &npc_id,
                "attack"
            )
        );
    }

    let enemy_dmg = roll(NPC_MIN, NPC_MAX, 2);
    let attacker_hp = {
        let mut players = state.players.lock().await;
        let player = players.get_mut(&username).unwrap();
        player.hp = (player.hp - enemy_dmg).max(0);
        player.hp
    };
    let target_hp = {
        let world_state = state.world_state.lock().await;
        world_state.npcs.get(&npc_id).map(|n| n.hp).unwrap_or(0)
    };

    if attacker_hp > 0 {
        {
            let mut players = state.players.lock().await;
            players.get_mut(&username).unwrap().combat_turn = Turn::Player;
        }
        log_output("INFO", "COMBAT_RESULT", serde_json::json!({
            "player": username, "npc": npc_id, "action": "enemy_attack",
            "damage": enemy_dmg, "player_hp": attacker_hp, "result": "hit"
        }));
        return format!(
            "OK {}",
            combat_json(
                "enemy",
                attacker_hp,
                target_hp,
                enemy_dmg,
                "combat",
                &npc_id,
                "attack"
            )
        );
    }

    respawn_player(&username, &npc_id, npc_max_hp, Arc::clone(&state)).await;
    format!(
        "OK {}",
        combat_json(
            "enemy", RESPAWN_HP, npc_max_hp, enemy_dmg, "defeat", &npc_id, "attack"
        )
    )
}

pub async fn defend(username: String, state: Arc<SharedState>) -> String {
    let (npc_id, player_room) = {
        let players = state.players.lock().await;
        let player = match players.get(&username) {
            Some(p) => p,
            None => return "ERR 407 NOT_IN_COMBAT".to_string(),
        };
        match &player.combat_target {
            Some(target) => (target.clone(), player.room.clone()),
            None => return "ERR 407 NOT_IN_COMBAT".to_string(),
        }
    };

    let npc_max_hp = {
        let world_data = state.world_data.lock().await;
        world_data
            .world
            .npcs
            .get(&npc_id)
            .map(|n| n.hp)
            .unwrap_or(MAX_HP)
    };

    let raw_dmg = roll(NPC_MIN, NPC_MAX, 3);
    let enemy_dmg = (raw_dmg * DEFEND_DAMAGE_PERCENT / 100).max(0);
    let riposte = (raw_dmg * RIPOSTE_PERCENT / 100).max(0);

    let attacker_hp = {
        let mut players = state.players.lock().await;
        let player = players.get_mut(&username).unwrap();
        player.hp = (player.hp - enemy_dmg).max(0);
        player.combat_turn = Turn::Player;
        player.hp
    };

    if attacker_hp <= 0 {
        let target_hp = {
            let world_state = state.world_state.lock().await;
            world_state.npcs.get(&npc_id).map(|n| n.hp).unwrap_or(0)
        };
        respawn_player(&username, &npc_id, npc_max_hp, Arc::clone(&state)).await;
        return format!(
            "OK {}",
            defend_json(RESPAWN_HP, target_hp, enemy_dmg, 0, "defeat", &npc_id)
        );
    }

    let target_hp = {
        let mut world_state = state.world_state.lock().await;
        let npc = world_state.npcs.get_mut(&npc_id).unwrap();
        npc.hp = (npc.hp - riposte).max(0);
        npc.hp
    };

    if target_hp <= 0 {
        {
            let mut players = state.players.lock().await;
            let player = players.get_mut(&username).unwrap();
            player.combat_turn = Turn::Player;
            player.combat_target = None;
        }
        log_output("INFO", "COMBAT_RESULT", serde_json::json!({
            "player": username, "npc": npc_id, "action": "defend",
            "damage": enemy_dmg, "counter": riposte, "result": "victory"
        }));
        broadcast_room(
            &player_room,
            &format!("EVT ROOM COMBAT {} defeated {}", username, npc_id),
            Arc::clone(&state),
        )
        .await;
        return format!(
            "OK {}",
            defend_json(attacker_hp, 0, enemy_dmg, riposte, "victory", &npc_id)
        );
    }

    log_output("INFO", "COMBAT_RESULT", serde_json::json!({
        "player": username, "npc": npc_id, "action": "defend",
        "damage": enemy_dmg, "counter": riposte, "target_hp": target_hp, "result": "combat"
    }));
    broadcast_room(
        &player_room,
        &format!(
            "EVT ROOM COMBAT {} parries {} and ripostes for {}",
            username, npc_id, riposte
        ),
        Arc::clone(&state),
    )
    .await;
    format!(
        "OK {}",
        defend_json(
            attacker_hp,
            target_hp,
            enemy_dmg,
            riposte,
            "combat",
            &npc_id
        )
    )
}

fn defend_json(
    attacker_hp: i32,
    target_hp: i32,
    damage: i32,
    counter: i32,
    status: &str,
    enemy: &str,
) -> String {
    serde_json::json!({
        "actor": "player",
        "attacker_hp": attacker_hp,
        "target_hp": target_hp,
        "damage": damage,
        "counter": counter,
        "status": status,
        "enemy": enemy,
        "action": "defend",
    })
    .to_string()
}

pub async fn flee(username: String, state: Arc<SharedState>) -> String {
    let (npc_id, player_room) = {
        let players = state.players.lock().await;
        let player = match players.get(&username) {
            Some(p) => p,
            None => return "ERR 407 NOT_IN_COMBAT".to_string(),
        };
        match &player.combat_target {
            Some(target) => (target.clone(), player.room.clone()),
            None => return "ERR 407 NOT_IN_COMBAT".to_string(),
        }
    };

    let npc_max_hp = {
        let world_data = state.world_data.lock().await;
        world_data
            .world
            .npcs
            .get(&npc_id)
            .map(|n| n.hp)
            .unwrap_or(MAX_HP)
    };

    let success = roll(1, 100, 4) <= FLEE_CHANCE_PERCENT;

    if success {
        let destination = {
            let world_data = state.world_data.lock().await;
            let exits: Vec<String> = world_data
                .world
                .rooms
                .get(&player_room)
                .map(|room| room.exits.values().cloned().collect())
                .unwrap_or_default();
            if exits.is_empty() {
                None
            } else {
                let idx = roll(0, exits.len() as i32 - 1, 5) as usize;
                Some(exits[idx].clone())
            }
        };

        let Some(destination) = destination else {
            return flee_failed(username, npc_id, npc_max_hp, state).await;
        };

        {
            let mut players = state.players.lock().await;
            let player = players.get_mut(&username).unwrap();
            player.room = destination.clone();
            player.combat_turn = Turn::Player;
            player.combat_target = None;
        }
        {
            let mut world_state = state.world_state.lock().await;
            if let Some(room) = world_state.room.get_mut(&player_room) {
                room.players.retain(|p| p != &username);
            }
            if let Some(room) = world_state.room.get_mut(&destination) {
                room.players.push(username.clone());
            }
        }
        broadcast_room(
            &player_room,
            &format!("EVT ROOM PRESENCE LEAVE {}", username),
            Arc::clone(&state),
        )
        .await;
        broadcast_room(
            &destination,
            &format!("EVT ROOM PRESENCE ENTER {}", username),
            Arc::clone(&state),
        )
        .await;
        broadcast_room(
            &player_room,
            &format!("EVT ROOM COMBAT {} fled from {}", username, npc_id),
            Arc::clone(&state),
        )
        .await;
        let attacker_hp = {
            let players = state.players.lock().await;
            players.get(&username).map(|p| p.hp).unwrap_or(0)
        };
        log_output("INFO", "COMBAT_RESULT", serde_json::json!({
            "player": username, "npc": npc_id, "action": "flee", "result": "fled", "room": destination
        }));
        return format!(
            "OK {}",
            serde_json::json!({
                "actor": "player",
                "attacker_hp": attacker_hp,
                "status": "fled",
                "enemy": npc_id,
                "room": destination,
                "action": "flee",
            })
        );
    }

    flee_failed(username, npc_id, npc_max_hp, state).await
}

async fn flee_failed(
    username: String,
    npc_id: String,
    npc_max_hp: i32,
    state: Arc<SharedState>,
) -> String {
    let enemy_dmg = roll(NPC_MIN, NPC_MAX, 6);
    let attacker_hp = {
        let mut players = state.players.lock().await;
        let player = players.get_mut(&username).unwrap();
        player.hp = (player.hp - enemy_dmg).max(0);
        player.combat_turn = Turn::Player;
        player.hp
    };
    let target_hp = {
        let world_state = state.world_state.lock().await;
        world_state.npcs.get(&npc_id).map(|n| n.hp).unwrap_or(0)
    };

    if attacker_hp <= 0 {
        respawn_player(&username, &npc_id, npc_max_hp, Arc::clone(&state)).await;
        return format!(
            "OK {}",
            combat_json(
                "enemy", RESPAWN_HP, target_hp, enemy_dmg, "defeat", &npc_id, "flee"
            )
        );
    }
    log_output("INFO", "COMBAT_RESULT", serde_json::json!({
        "player": username, "npc": npc_id, "action": "flee_failed",
        "damage": enemy_dmg, "player_hp": attacker_hp, "result": "hit"
    }));
    format!(
        "OK {}",
        combat_json(
            "enemy",
            attacker_hp,
            target_hp,
            enemy_dmg,
            "combat",
            &npc_id,
            "flee_failed"
        )
    )
}

pub async fn use_item(username: String, item_name_or_id: &str, state: Arc<SharedState>) -> String {
    let (item_id, heal) = {
        let players = state.players.lock().await;
        let player = match players.get(&username) {
            Some(p) => p,
            None => return "ERR 404 ITEM_NOT_IN_INVENTORY".to_string(),
        };
        let world_data = state.world_data.lock().await;
        let resolved = player.inventory.iter().find(|owned| {
            owned.as_str() == item_name_or_id
                || world_data
                    .world
                    .items
                    .get(owned.as_str())
                    .map(|item| item.name == item_name_or_id)
                    .unwrap_or(false)
        });
        match resolved {
            Some(id) => {
                let heal = world_data
                    .world
                    .items
                    .get(id.as_str())
                    .map(|item| item.heal)
                    .unwrap_or(0);
                (id.clone(), heal)
            }
            None => return "ERR 404 ITEM_NOT_IN_INVENTORY".to_string(),
        }
    };

    if heal <= 0 {
        return "ERR 409 ITEM_NOT_USABLE".to_string();
    }

    let hp = {
        let mut players = state.players.lock().await;
        let player = players.get_mut(&username).unwrap();
        if let Some(pos) = player.inventory.iter().position(|i| i == &item_id) {
            player.inventory.remove(pos);
        }
        player.hp = (player.hp + heal).min(MAX_HP);
        player.hp
    };

    log_output("INFO", "COMBAT_RESULT", serde_json::json!({
        "player": username, "action": "use_item", "item": item_id, "healed": heal, "hp": hp
    }));

    format!(
        "OK {}",
        serde_json::json!({
            "used": item_id,
            "healed": heal,
            "hp": hp,
            "max_hp": MAX_HP,
            "status": "healed",
        })
    )
}

pub async fn status(username: String, state: Arc<SharedState>) -> String {
    let players = state.players.lock().await;
    let player = players.get(&username).unwrap();
    let label = if player.hp <= 0 {
        "dead"
    } else if player.hp < MAX_HP / 2 {
        "wounded"
    } else {
        "healthy"
    };
    let in_combat = player.combat_target.is_some();
    format!(
        "OK {}",
        serde_json::json!({
            "hp": player.hp,
            "max_hp": MAX_HP,
            "status": label,
            "in_combat": in_combat,
        })
    )
}
