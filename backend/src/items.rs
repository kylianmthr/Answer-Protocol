use std::sync::Arc;

use crate::broadcast::broadcats_room_except;
use crate::state::{SharedState};
use crate::logs_format::log_output;

pub async fn take(
    player_name: String,
    item_name_or_id: String,
    state: Arc<SharedState>,
) -> Result<String, String> {
    let mut world_state = state.world_state.lock().await;
	let world_data = state.world_data.lock().await;
    let mut players = state.players.lock().await;
    let player = players
        .get_mut(&player_name)
        .ok_or_else(|| format!("Player '{}' not found", player_name))?;
    let room = world_state
        .room
        .get_mut(player.room.as_str())
        .ok_or_else(|| format!("Room '{}' not found", player.room))?;

	let room_id = player.room.clone();
	let item_id: String = if room.items.contains(&item_name_or_id) {
		item_name_or_id.clone()
	} else {
		world_data
			.world
			.items
			.iter()
			.find(|(id, item)| item.name == item_name_or_id && room.items.contains(id))
			.map(|(id, _)| id.clone())
			.ok_or_else(|| {
				format!("Item '{}' not found in room '{}'", item_name_or_id, room_id)
			})?
	};

	room.items.retain(|item| item != &item_id);
	player.inventory.push(item_id.clone());
	drop(players);
	broadcats_room_except(&room_id, &player_name, "OK refresh",
	Arc::clone(&state),).await;
	log_output(
		"INFO",
		"TAKEN",
		serde_json::json!({
			"player": &player_name,
			"ITEM_ID": &item_id,
			"ROOM_ID": &room_id
		}),
	);

	Ok(format!("OK taken={}", item_id))
}

pub async fn drop_item(
    player_name: String,
    item_name_or_id: String,
    state: Arc<SharedState>,
) -> Result<String, String> {
    let mut world_state = state.world_state.lock().await;
    let world_data = state.world_data.lock().await;
    let mut players = state.players.lock().await;
    let player = players
        .get_mut(&player_name)
        .ok_or_else(|| format!("Player '{}' not found", player_name))?;
    let room = world_state
        .room
        .get_mut(player.room.as_str())
        .ok_or_else(|| format!("Room '{}' not found", player.room))?;

    let room_id = player.room.clone();

    let item_id: String = if player.inventory.contains(&item_name_or_id) {
        item_name_or_id.clone()
    } else {
        world_data
            .world
            .items
            .iter()
            .find(|(id, item)| item.name == item_name_or_id && player.inventory.contains(id))
            .map(|(id, _)| id.clone())
            .ok_or_else(|| "ERR 404 ITEM_NOT_IN_INVENTORY\n".to_string())?
    };

    player.inventory.retain(|item| item != &item_id);
    room.items.push(item_id.clone());
	drop(players);
	broadcats_room_except(&room_id, &player_name, "refresh",
	Arc::clone(&state),).await;
    log_output(
        "INFO",
        "DROPPED",
        serde_json::json!({
            "player": &player_name,
            "ITEM_ID": &item_id,
            "ROOM_ID": &room_id
        }),
    );

    Ok(format!("OK dropped={}", item_id))
}

// pub async fn inventory(player_name: String, state: Arc<SharedState>) -> Result<String, String> {
//     let players = state.players.lock().await;
//     let player = players
//         .get(&player_name)
//         .ok_or_else(|| format!("Player '{}' not found", player_name))?;
// 	log_output("INFO", "INVENTORY", serde_json::json!({
// 							"player": player_name
// 						}));
// 	Ok(format!("OK inventory={:?}", player.inventory))
// }
