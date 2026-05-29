use crate::state::SharedState;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MoveError {
    #[error("INVALID_DIRECTION")]
    InvalidDirection,
}

pub async fn move_cmd(
    username: String,
    direction: String,
    state: Arc<SharedState>,
) -> Result<String, MoveError> {
    let mut players = state.players.lock().await;
    let player = players.get_mut(&username).unwrap();
    if direction != "north" && direction != "south" && direction != "east" && direction != "west" {
        return Err(MoveError::InvalidDirection);
    }
    let world_data = state.world_data.lock().await;
    let room = world_data.world.rooms.get(player.room.as_str()).unwrap();
    if room.exits.get(direction.as_str()).is_none() {
        return Err(MoveError::InvalidDirection);
    }
    let mut world_state = state.world_state.lock().await;
    let next_room = world_state
        .room
        .get_mut(room.exits.get(direction.as_str()).unwrap())
        .unwrap();
    next_room.players.push(username.clone());
    let current_room = world_state.room.get_mut(player.room.as_str()).unwrap();
    current_room.players.retain(|p| p != &username);
    player.room = room.exits.get(direction.as_str()).unwrap().to_string();
    Ok(room.exits.get(direction.as_str()).unwrap().to_string())
}
