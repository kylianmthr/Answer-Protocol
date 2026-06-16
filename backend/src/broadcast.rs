use crate::state::SharedState;
use std::sync::Arc;

pub async fn broadcast_room(room_id: &str, message: &str, state: Arc<SharedState>) {
    let players = state.players.lock().await;
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

pub async fn broadcast_group(group_id: &str, message: &str, state: Arc<SharedState>) {
    let groups = state.groups.lock().await;
    if let Some(group) = groups.get(group_id) {
        group.members.iter().for_each(|member| {
            let _ = member.tx.send(message.to_string());
        });
    }
}
