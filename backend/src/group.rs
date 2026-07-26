use crate::state::{Group, SharedState};
use std::sync::Arc;

/// Create a new group led by `owner_name`. Per RFC 5.3.1 the client sends no
/// group name; the server generates the group-id and returns it to the caller.
pub async fn group_create(owner_name: &str, state: Arc<SharedState>) -> Result<String, String> {
    let mut groups = state.groups.lock().await;
    let mut players = state.players.lock().await;
    let player = players
        .get_mut(owner_name)
        .ok_or_else(|| "PLAYER_NOT_FOUND".to_string())?;
    if player.group.is_some() {
        return Err("ALREADY_IN_GROUP".to_string());
    }
    // A player can lead at most one group at a time, so this id is unique.
    let group_id = format!("grp.{}", owner_name);
    if groups.contains_key(&group_id) {
        return Err("ALREADY_EXIST".to_string());
    }
    player.group = Some(group_id.clone());
    let mut group = Group::new(&group_id, owner_name);
    group.add_member(player.clone());
    groups.insert(group_id.clone(), group);
    Ok(group_id)
}

pub async fn group_invite(
    group_name: &str,
    player_name: &str,
    owner_name: &str,
    state: Arc<SharedState>,
) -> Result<(), String> {
    let mut groups = state.groups.lock().await;
    let mut players = state.players.lock().await;

    if let Some(group) = groups.get(group_name) {
        let leader = group.leader.clone();
        let group_snapshot = group.clone();
        if let Some(player) = players.get_mut(player_name) {
            if player.group.is_some() {
                return Err("ALREADY_IN_GROUP".to_string());
            }
            player.invitations.push(group_snapshot);
            // Per RFC 6.2.3 the invite event carries the leader's name, which is
            // what the invitee passes to GROUP JOIN.
            player
                .tx
                .send(format!("EVT GROUP INVITE {}", leader))
                .map_err(|_| "Failed to send invitation".to_string())?;
            Ok(())
        } else {
            Err("PLAYER_NOT_FOUND".to_string())
        }
    } else {
        Err("GROUP_NOT_FOUND".to_string())
    }
}

pub async fn group_leave(player_name: &str, state: Arc<SharedState>) -> Result<(), String> {
    let mut groups = state.groups.lock().await;
    let mut players = state.players.lock().await;

    let player = players
        .get_mut(player_name)
        .ok_or_else(|| "PLAYER_NOT_FOUND".to_string())?;
    let group_name = player
        .group
        .take()
        .ok_or_else(|| "NOT_IN_GROUP".to_string())?;
    if let Some(group) = groups.get_mut(&group_name) {
        group.remove_member(player_name);
        for member in &group.members {
            let _ = member.tx.send(format!("EVT GROUP LEAVE {}", player_name));
        }
    }
    Ok(())
}

/// Join the group led by `leader_name` (RFC 5.3.3: `GROUP JOIN <leader-name>`).
/// The player must have been invited to that group. Returns the group-id.
pub async fn group_join(
    leader_name: &str,
    player_name: &str,
    state: Arc<SharedState>,
) -> Result<String, String> {
    let mut groups = state.groups.lock().await;
    let mut players = state.players.lock().await;

    // Resolve the group from its leader's name.
    let group_id = groups
        .iter()
        .find(|(_, group)| group.leader == leader_name)
        .map(|(id, _)| id.clone())
        .ok_or_else(|| "GROUP_NOT_FOUND".to_string())?;

    let player = players
        .get_mut(player_name)
        .ok_or_else(|| "PLAYER_NOT_FOUND".to_string())?;
    if player.group.is_some() {
        return Err("ALREADY_IN_GROUP".to_string());
    }
    if player
        .invitations
        .iter()
        .all(|invitation| invitation.id != group_id)
    {
        return Err("NO_INVITATION".to_string());
    }
    player.group = Some(group_id.clone());
    player
        .invitations
        .retain(|invitation| invitation.id != group_id);
    let member = player.clone();

    let group = groups.get_mut(&group_id).unwrap();
    group.add_member(member);
    for member in &group.members {
        let _ = member.tx.send(format!("EVT GROUP JOIN {}", player_name));
    }
    Ok(group_id)
}
