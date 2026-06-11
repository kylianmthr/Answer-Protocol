use crate::state::SharedState;
use std::sync::Arc;

pub async fn broadcast_room(room_id: &str, message: &str, state: Arc<SharedState>) {
    let players = state.players.lock().await;
    println!("Broadcasting to room {}: {}", room_id, message);
    players.iter().for_each(|(_, player)| {
        if player.room == room_id {
            let _ = player.tx.send(message.to_string());
        }
    });
}

pub async fn broadcast_global(message: &str, state: Arc<SharedState>) {
    let players = state.players.lock().await;
    println!("Broadcasting globally: {}", message);
    players.iter().for_each(|(_, player)| {
        let _ = player.tx.send(message.to_string());
    });
}
