use std::sync::Arc;

use crate::state::SharedState;

pub async fn who(username: String, state: Arc<SharedState>) -> String {
    let world_state = state.world_state.lock().await;
    let players = state.players.lock().await;
    let player_room = world_state
        .room
        .get(&players.get(&username).unwrap().room)
        .unwrap();
    let room_id = player_room.id.clone();
    let players_in_room = player_room.players.clone();
    drop(world_state);
    let res = serde_json::json!({
        "room": players_in_room,
        "server": players.len(),
    });
    format!("OK {}", res)
}
