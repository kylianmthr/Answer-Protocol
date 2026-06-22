use crate::state::WorldData;
use std::collections::HashMap;
use thiserror::Error;
use validator::ValidationError;

pub fn validate_exits(exits: &HashMap<String, String>) -> Result<(), ValidationError> {
    let valid_directions = ["north", "south", "east", "west"];

    for key in exits.keys() {
        if !valid_directions.contains(&key.as_str()) {
            return Err(ValidationError::new("invalid_exit_direction"));
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum WorldError {
    #[error("INVALID_EXIT_REF")]
    InvalidExitRef,
    #[error("INVALID_ITEM_REF")]
    InvalidItemRef,
    #[error("INVALID_NPC_REF")]
    InvalidNpcRef,
    #[error("INVALID_ROOM_REF")]
    InvalidRoomRef,
}

pub fn validate_yaml(world_data: &WorldData) -> Result<(), WorldError> {
    for room in world_data.world.rooms.values() {
        for exit in room.exits.values() {
            if !world_data.world.rooms.contains_key(exit) {
                return Err(WorldError::InvalidExitRef);
            }
        }
        for item in &room.items {
            if !world_data.world.items.contains_key(item) {
                return Err(WorldError::InvalidItemRef);
            }
        }
        for npc in &room.npcs {
            if !world_data.world.npcs.contains_key(npc) {
                return Err(WorldError::InvalidNpcRef);
            }
        }
    }
    if !world_data
        .world
        .rooms
        .contains_key(world_data.world.initial_room.as_str())
    {
        return Err(WorldError::InvalidRoomRef);
    }
    for npc in world_data.world.npcs.values() {
        if !world_data.world.rooms.contains_key(npc.room.as_str()) {
            return Err(WorldError::InvalidRoomRef);
        }
    }

    for npc in world_data.world.npcs.values() {
        if let Some(quest) = &npc.quest {
            if !world_data.world.items.contains_key(&quest.reward) {
                return Err(WorldError::InvalidItemRef);
            }
        }
    }
    Ok(())
}
