use std::sync::Arc;

use crate::state::SharedState;

pub async fn take(
    player_name: String,
    item_name_or_id: String,
    state: Arc<SharedState>,
) -> Result<String, String> {
    let mut world_state = state.world_state.lock().await;
    let mut players = state.players.lock().await;
    let player = players
        .get_mut(&player_name)
        .ok_or_else(|| format!("Player '{}' not found", player_name))?;
    let room = world_state
        .room
        .get_mut(player.room.as_str())
        .ok_or_else(|| format!("Room '{}' not found", player.room))?;

    if !room.items.contains(&item_name_or_id)
        && !room.items.iter().any(|item| item == &item_name_or_id)
    {
        return Err(format!(
            "Item '{}' not found in room '{}'",
            item_name_or_id, player.room
        ));
    }

    if !room.items.contains(&item_name_or_id) {
        let item_name_or_id = room
            .items
            .iter()
            .find(|item| item == &&item_name_or_id)
            .ok_or_else(|| {
                format!(
                    "Item '{}' not found in room '{}'",
                    item_name_or_id, player.room
                )
            })?
            .clone();
        room.items.retain(|item| item != &item_name_or_id);
        player.inventory.push(item_name_or_id.clone());
        Ok(format!("OK taken={}", item_name_or_id))
    } else {
        room.items.retain(|item| item != &item_name_or_id);
        player.inventory.push(item_name_or_id.clone());
        Ok(format!("OK taken={}", item_name_or_id))
    }
}

pub async fn drop(
    player_name: String,
    item_name_or_id: String,
    state: Arc<SharedState>,
) -> Result<String, String> {
    let mut world_state = state.world_state.lock().await;
    let mut players = state.players.lock().await;
    let player = players
        .get_mut(&player_name)
        .ok_or_else(|| format!("Player '{}' not found", player_name))?;
    let room = world_state
        .room
        .get_mut(player.room.as_str())
        .ok_or_else(|| format!("Room '{}' not found", player.room))?;

    if !player.inventory.contains(&item_name_or_id) {
        return Err("ERR 404 ITEM_NOT_IN_INVENTORY\n".to_string());
    }

    player.inventory.retain(|item| item != &item_name_or_id);
    room.items.push(item_name_or_id.clone());
    Ok(format!("OK dropped={}", item_name_or_id))
}

pub async fn inventory(player_name: String, state: Arc<SharedState>) -> Result<String, String> {
    let players = state.players.lock().await;
    let player = players
        .get(&player_name)
        .ok_or_else(|| format!("Player '{}' not found", player_name))?;
    Ok(format!("OK inventory={:?}", player.inventory))
}
