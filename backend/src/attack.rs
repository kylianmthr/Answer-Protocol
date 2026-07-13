use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::broadcast::broadcast_room;
use crate::state::{SharedState, Turn};

// Combat design (see README "Combat System"):
// Combat is turn-based: each ATTACK resolves exactly ONE action. On the player's
// turn the player strikes; the turn then flips to the enemy, whose retaliation
// is resolved by the next ATTACK. This keeps the two attacks separate instead of
// a simultaneous exchange. Damage is random and the ranges are close enough that
// a fight from full HP is a real coin-flip. On death the player respawns at full
// HP in the starting room and the enemy is restored to full, so every attempt is
// a fresh duel.
const PLAYER_MIN: i32 = 20;
const PLAYER_MAX: i32 = 30;
const NPC_MIN: i32 = 20;
const NPC_MAX: i32 = 30;
pub const MAX_HP: i32 = 100;

// dependency-free inclusive random roll (time-seeded xorshift, salted so two
// rolls in the same call don't collide)
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
) -> String {
    serde_json::json!({
        "actor": actor,
        "attacker_hp": attacker_hp,
        "target_hp": target_hp,
        "damage": damage,
        "status": status,
        "enemy": enemy,
    })
    .to_string()
}

pub async fn attack(username: String, npc_name_or_id: &str, state: Arc<SharedState>) -> String {
    // current room of the attacker
    let player_room = {
        let players = state.players.lock().await;
        match players.get(&username) {
            Some(player) => player.room.clone(),
            None => return "ERR 404 NPC_NOT_FOUND".to_string(),
        }
    };

    // resolve the target NPC (by id or display name) inside the player's room
    let (npc_id, hostile, initial_room, npc_max_hp) = {
        let world_data = state.world_data.lock().await;
        let resolved = world_data.world.npcs.iter().find(|(id, npc)| {
            (id.as_str() == npc_name_or_id || npc.name == npc_name_or_id) && npc.room == player_room
        });
        match resolved {
            Some((id, npc)) => (
                id.clone(),
                npc.hostile,
                world_data.world.initial_room.clone(),
                npc.hp,
            ),
            None => return "ERR 404 NPC_NOT_FOUND".to_string(),
        }
    };

    if !hostile {
        return "ERR 405 NPC_NOT_HOSTILE".to_string();
    }

    // whose turn is it? read (and don't yet flip) the player's combat turn
    let turn = {
        let players = state.players.lock().await;
        players
            .get(&username)
            .map(|p| p.combat_turn.clone())
            .unwrap_or(Turn::Player)
    };

    if turn == Turn::Player {
        // --- PLAYER'S TURN: strike the enemy ---
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
            // enemy down: combat over, reset the turn for the next fight
            {
                let mut players = state.players.lock().await;
                players.get_mut(&username).unwrap().combat_turn = Turn::Player;
            }
            broadcast_room(
                &player_room,
                &format!("EVT ROOM COMBAT {} defeated {}", username, npc_id),
                Arc::clone(&state),
            )
            .await;
            return format!(
                "OK {}",
                combat_json("player", attacker_hp, 0, player_dmg, "victory", &npc_id)
            );
        }

        // enemy survives: it's now the enemy's turn
        {
            let mut players = state.players.lock().await;
            players.get_mut(&username).unwrap().combat_turn = Turn::Enemy;
        }
        return format!(
            "OK {}",
            combat_json("player", attacker_hp, target_hp, player_dmg, "combat", &npc_id)
        );
    }

    // --- ENEMY'S TURN: it strikes back ---
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
        // back to the player's turn
        {
            let mut players = state.players.lock().await;
            players.get_mut(&username).unwrap().combat_turn = Turn::Player;
        }
        return format!(
            "OK {}",
            combat_json("enemy", attacker_hp, target_hp, enemy_dmg, "combat", &npc_id)
        );
    }

    // player down: restore the enemy and respawn the player, full HP, at the
    // starting room so the duel can be retried from scratch
    {
        let mut world_state = state.world_state.lock().await;
        world_state.npcs.get_mut(&npc_id).unwrap().hp = npc_max_hp;
    }
    let old_room = {
        let mut players = state.players.lock().await;
        let player = players.get_mut(&username).unwrap();
        let old = player.room.clone();
        player.hp = MAX_HP;
        player.room = initial_room.clone();
        player.combat_turn = Turn::Player;
        old
    };
    {
        let mut world_state = state.world_state.lock().await;
        if let Some(room) = world_state.room.get_mut(&old_room) {
            room.players.retain(|p| p != &username);
        }
        if let Some(room) = world_state.room.get_mut(&initial_room) {
            room.players.push(username.clone());
        }
    }
    broadcast_room(
        &player_room,
        &format!("EVT ROOM COMBAT {} was defeated by {}", username, npc_id),
        Arc::clone(&state),
    )
    .await;
    format!(
        "OK {}",
        combat_json("enemy", MAX_HP, npc_max_hp, enemy_dmg, "defeat", &npc_id)
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
    format!(
        "OK {}",
        serde_json::json!({
            "hp": player.hp,
            "max_hp": MAX_HP,
            "status": label,
        })
    )
}
