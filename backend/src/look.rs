use crate::SharedState;
use std::sync::Arc;

pub async fn look(username: String, state: Arc<SharedState>) -> String {
    let room_id = {
        let players = state.players.lock().await;
        players.get(&username).unwrap().room.clone()
    };
    let (name, description, exits) = {
        let world_data = state.world_data.lock().await;
        let room = world_data.world.rooms.get(room_id.as_str()).unwrap();
        (
            room.name.clone(),
            room.description.clone(),
            room.exits.clone(),
        )
    };
    let (players_in_room, items, npcs) = {
        let world_state = state.world_state.lock().await;
        let room = world_state.room.get(room_id.as_str()).unwrap();
        (
            room.players.clone(),
            room.items.clone(),
            room.npcs.clone(),
        )
    };
    serde_json::json!({
        "room": {
            "id": room_id,
            "name": name,
            "description": description,
            "exits": exits,
        },
        "players": players_in_room,
        "items": items,
        "npcs": npcs,
    })
    .to_string()
}
