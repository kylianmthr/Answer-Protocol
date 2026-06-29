use crate::SharedState;
use std::sync::Arc;

#[derive(Serialize)]
pub struct LookSer {
    pub id: String,
    pub items: Vec<String>,
    pub npcs: Vec<String>,
    pub players: Vec<String>,
    }

pub async fn look(username: String, state: Arc<SharedState>) -> String {
    let mut world_state = state.world_state.lock().await;
    let players = state.players.lock().await;
    let player = players.get(&username).unwrap();
    let room = world_state.room.get_mut(player.room.as_str()).unwrap();
	room.inventory = player.inventory.clone();
	let output_look = LookSer {
        id: room.id.clone(),
        items: room.items.clone(),
        ncps: room.ncps.clone(),
        players: room.players.clone(),
        };
	return serde_json::to_string(&output_look).unwrap();
}
