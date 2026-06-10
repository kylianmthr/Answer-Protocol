use crate::{broadcast::broadcast_room, state::SharedState};
use std::sync::Arc;

pub async fn chat_room(message: String, username: String, state: Arc<SharedState>) {
    let world_state = state.world_state.lock().await;
    let player_room = world_state
        .room
        .get(&state.players.lock().await.get(&username).unwrap().room)
        .unwrap();
    let room_id = player_room.id.clone();
    drop(world_state);
    broadcast_room(
        room_id.as_str(),
        format!("EVT ROOM CHAT {} {}", username, message).as_str(),
        state.clone(),
    )
    .await;
}
