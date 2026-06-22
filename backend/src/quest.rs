use std::sync::Arc;

use crate::state::SharedState;

pub async fn quest(username: String, npc_name_or_id: &str, state: Arc<SharedState>) -> String {
    let world_data = state.world_data.lock().await;
    let npc_data = world_data.world.npcs.get(npc_name_or_id).or_else(|| {
        world_data
            .world
            .npcs
            .values()
            .find(|npc| npc.name == npc_name_or_id)
    });
    let npc = match npc_data {
        Some(npc) => npc,
        None => return "ERR 404 NPC not found".to_string(),
    };
    let quest = match &npc.quest {
        Some(quest) => quest,
        None => return "ERR 404 NPC has no quest".to_string(),
    };
    let mut players = state.players.lock().await;
    let player = players.get_mut(&username).unwrap();
    player.quests.push(quest.clone());
    format!("OK {}", serde_json::to_string(&quest).unwrap())
}

pub async fn get_quests(username: String, state: Arc<SharedState>) -> String {
    let players = state.players.lock().await;
    let player = players.get(&username).unwrap();
    format!("OK {}", serde_json::to_string(&player.quests).unwrap())
}
