use crate::state::{Group, SharedState};
use std::sync::Arc;

pub async fn group_create(
    group_name: &str,
    owner_name: &str,
    state: Arc<SharedState>,
) -> Result<(), String> {
    let mut groups = state.groups.lock().await;
    let mut players = state.players.lock().await;
    if group_name.is_empty() {
        return Err("EMPTY_NAME".to_string());
    }
    if groups.contains_key(group_name) {
        return Err("ALREADY_EXIST".to_string());
    }
    let player = players
        .get_mut(owner_name)
        .ok_or_else(|| "PLAYER_NOT_FOUND".to_string())?;
    if player.group.is_some() {
        return Err("ALREADY_IN_GROUP".to_string());
    }
    groups.insert(group_name.to_string(), Group::new(group_name));
    player.group = Some(group_name.to_string());
    groups
        .get_mut(group_name)
        .unwrap()
        .add_member(player.clone());
    Ok(())
}

pub async fn group_invite(
    group_name: &str,
    player_name: &str,
    owner_name: &str,
    state: Arc<SharedState>,
) -> Result<(), String> {
    let mut groups = state.groups.lock().await;
    let mut players = state.players.lock().await;

    if let Some(group) = groups.get_mut(group_name) {
        if let Some(player) = players.get_mut(player_name) {
            if player.group.is_some() {
                return Err("ALREADY_IN_GROUP".to_string());
            }
            player.invitations.push(group.clone());
            player
                .tx
                .send(format!("EVT GROUP INVITE {}", group_name))
                .map_err(|_| "Failed to send invitation".to_string())?;
            Ok(())
        } else {
            Err("PLAYER_NOT_FOUND".to_string())
        }
    } else {
        Err("GROUP_NOT_FOUND".to_string())
    }
}

pub async fn group_accept(
    group_name: &str,
    player_name: &str,
    state: Arc<SharedState>,
) -> Result<(), String> {
    let mut groups = state.groups.lock().await;
    let mut players = state.players.lock().await;

    if let Some(group) = groups.get_mut(group_name) {
        if let Some(player) = players.get_mut(player_name) {
            if player.group.is_some() {
                return Err("ALREADY_IN_GROUP".to_string());
            }
            if player
                .invitations
                .iter()
                .all(|invitation| invitation.id != group_name)
            {
                return Err("NO_INVITATION".to_string());
            }
            player.group = Some(group_name.to_string());
            group.add_member(player.clone());
            for player in &group.members {
                let _ = player.tx.send(format!("EVT GROUP JOIN {}", player_name));
            }
            player
                .invitations
                .retain(|invitation| invitation.id != group_name);
            Ok(())
        } else {
            Err("PLAYER_NOT_FOUND".to_string())
        }
    } else {
        Err("GROUP_NOT_FOUND".to_string())
    }
}
