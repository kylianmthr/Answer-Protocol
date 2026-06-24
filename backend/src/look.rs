use crate::SharedState;
use std::sync::Arc;

pub async fn look(username: String, state: Arc<SharedState>) -> String {
    let mut world_state = state.world_state.lock().await;
    let players = state.players.lock().await;
    let player = players.get(&username).unwrap();
    let room = world_state.room.get_mut(player.room.as_str()).unwrap();
	room.inventory = player.inventory.clone();
	return serde_json::to_string(&room).unwrap();
}
