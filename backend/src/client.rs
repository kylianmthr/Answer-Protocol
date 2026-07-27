use crate::attack::attack;
use crate::attack::defend;
use crate::attack::flee;
use crate::attack::status;
use crate::attack::use_item;
use crate::broadcast::broadcast_global;
use crate::broadcast::broadcast_group;
use crate::chat::chat_room;
use crate::flood_systeme::command_check_flooding;
use crate::group::group_create;
use crate::group::group_invite;
use crate::group::group_join;
use crate::group::group_leave;
use crate::items::take;
use crate::look::look;
use crate::move_cmd::move_cmd;
use crate::logs_format;
use crate::quest::get_quests;
use crate::quest::quest;
use crate::state::Player;
use crate::state::SharedState;
use crate::talk::talk;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc;

const UNAUTHENTICATED: &str = "unauthenticated";

#[derive(Debug, Error)]
enum UserError {
    #[error("500 INVALID_USERNAME")]
    InvalidUsername,
    #[error("201 NAME_IN_USE")]
    AlreadyExist,
    #[error("500 BAD_PREFIX")]
    BadPrefix,
    #[error("500 INVALID_READ")]
    InvalidRead,
    #[error("500 INTERNAL_ERROR")]
    Internal,
}

fn group_err_line(e: &str) -> String {
    match e {
        "ALREADY_IN_GROUP" => "ERR 402 ALREADY_IN_GROUP".to_string(),
        "NOT_IN_GROUP" => "ERR 401 NOT_IN_GROUP".to_string(),
        "NO_INVITATION" => "ERR 403 NO_INVITATION".to_string(),
        "GROUP_NOT_FOUND" => "ERR 410 GROUP_NOT_FOUND".to_string(),
        other => format!("ERR 500 {}", other),
    }
}

async fn log_format(write: &mut OwnedWriteHalf, player: &str, response: &str) -> std::io::Result<()> {
    let level = if response.starts_with("ERR") { "WARN" } else { "INFO" };
    logs_format::log_output(
        level,
        "RESPONSE",
        serde_json::json!({
            "player": player,
            "response": response,
        }),
    );
    write.write_all(format!("{}\n", response).as_bytes()).await
}

async fn verify_authentication(
    line: Option<String>,
    state: Arc<SharedState>,
) -> Result<String, UserError> {
    let line = line.ok_or(UserError::InvalidRead)?;
    logs_format::log_output(
        "INFO",
        "COMMAND",
        serde_json::json!({
            "player": UNAUTHENTICATED,
            "cmd": "CONNECT",
            "line": line,
        }),
    );
    if let Some(username) = line.strip_prefix("CONNECT ") {
        if username.trim().is_empty() {
            return Err(UserError::InvalidUsername);
        }
        let players = state.players.lock().await;
        if players.get(username).is_some() {
            return Err(UserError::AlreadyExist);
        }
        return Ok(username.to_string());
    }
    Err(UserError::BadPrefix)
}

async fn add_player(
    username: String,
    state: Arc<SharedState>,
    tx: mpsc::UnboundedSender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut players = state.players.lock().await;
    let mut world_state = state.world_state.lock().await;
    let world_data = state.world_data.lock().await;
    let initial_room_state = world_state
        .room
        .get_mut(world_data.world.initial_room.as_str())
        .unwrap();
    let player = Player::new(&username, world_data.world.initial_room.as_str(), tx)?;
    players.insert(username.clone(), player.clone());
    initial_room_state.players.push(username.clone());
    Ok(())
}

async fn remove_player(username: &str, state: Arc<SharedState>) {
    let mut world_state = state.world_state.lock().await;
    let mut players = state.players.lock().await;
    let player = players.get(username).unwrap();
    let room = world_state.room.get_mut(player.room.as_str()).unwrap();
    room.players.retain(|p| p != username);
    players.remove(username);
    drop(players);
    drop(world_state);
    let maxima: Vec<(String, i32)> = {
        let world_data = state.world_data.lock().await;
        world_data
            .world
            .npcs
            .iter()
            .map(|(id, npc)| (id.clone(), npc.hp))
            .collect()
    };
    let mut world_state = state.world_state.lock().await;
    for (id, max_hp) in maxima {
        if let Some(npc) = world_state.npcs.get_mut(&id)
            && npc.hp <= 0 {
                npc.hp = max_hp;
            }
    }
}

async fn get_stats(state: &Arc<SharedState>) {
	let count_trafic = state.players.lock().await.len();
	broadcast_global(&format!("EVT STATS players={}", count_trafic), Arc::clone(state)).await;
}

async fn handle_commands(
    mut lines: Lines<BufReader<OwnedReadHalf>>,
    mut write: OwnedWriteHalf,
    mut rx: mpsc::UnboundedReceiver<String>,
    username: String,
    ip: String,
    state: Arc<SharedState>,
) {
    let reason;
    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        let mut parts = line.splitn(2, ' ');
                        let command = parts.next().unwrap_or("");
                        let args = parts.next().unwrap_or("").trim();
                        logs_format::log_output("INFO", "COMMAND", serde_json::json!({
                            "player": username, "cmd": command, "arg": args
                        }));
						command_check_flooding(&username, &state).await;
                        match command {
                            "LOOK" => {
                                let response = format!("OK {}", look(username.clone(), Arc::clone(&state)).await);
                                log_format(&mut write, &username, &response).await.expect("Can't send look response");
                            },
                            "MOVE" => {
                                match move_cmd(username.clone(), args.to_string(), Arc::clone(&state)).await {
                                    Ok(new_room) => {
                                        let response = format!("OK room={}", new_room);
                                        log_format(&mut write, &username, &response).await.expect("Can't send move response");
                                    },
                                    Err(e) => {
                                        let response = format!("ERR {}", e);
                                        log_format(&mut write, &username, &response).await.expect("Can't send move error response");
                                    }
                                }
                            },
                            "QUIT" => {
                                log_format(&mut write, &username, "OK bye").await.expect("Can't send goodbye message");
                                reason = "quit";
                                break;
                            },
                            "CHAT" => {
                                let scope = args.split(' ').next().unwrap_or("");
                                match scope {
                                    "ROOM" => {
                                        let message = args.strip_prefix("ROOM ").unwrap_or("").trim();
                                        chat_room(message.to_string(), username.clone(), Arc::clone(&state)).await;
                                        log_format(&mut write, &username, "OK").await.expect("Can't send chat response");
                                    },
                                    "GLOBAL" => {
                                        let message = args.strip_prefix("GLOBAL ").unwrap_or("").trim();
                                        broadcast_global(format!("EVT GLOBAL CHAT {} {}", username, message).as_str(), Arc::clone(&state)).await;
                                        log_format(&mut write, &username, "OK").await.expect("Can't send chat response");
                                    },
                                    "GROUP" => {
                                        let message = args.strip_prefix("GROUP ").unwrap_or("").trim();
                                        let players = state.players.lock().await;
                                        let group_id = players.get(&username).and_then(|player| player.group.clone());
                                        drop(players);
                                        broadcast_group(group_id.as_deref().unwrap_or(""), format!("EVT GROUP CHAT {} {}", username, message).as_str(), Arc::clone(&state)).await;
                                        log_format(&mut write, &username, "OK").await.expect("Can't send chat response");
                                    },
                                    _ => {
                                        log_format(&mut write, &username, "ERR 400 UNKNOWN_SCOPE").await.expect("Can't send unknown scope error");
                                    }
                                }
                            },
                            "WHO" => {
                                let response = crate::who::who(username.clone(), Arc::clone(&state)).await;
                                log_format(&mut write, &username, &response).await.expect("Can't send who response");
                            },
                            "TALK" => {
                                if args.is_empty() {
                                    log_format(&mut write, &username, "ERR 400 MISSING_NPC_NAME").await.expect("Can't send missing NPC name error");
                                    continue;
                                }
                                let npc_details = talk(args, Arc::clone(&state)).await;
                                if let Ok(details) = npc_details {
                                    let response = format!("OK {}", details);
                                    log_format(&mut write, &username, &response).await.expect("Can't send talk response");
                                } else {
                                    log_format(&mut write, &username, "ERR 404 NPC_NOT_FOUND").await.expect("Can't send talk error response");
                                }
                            },
                            "TAKE" => {
                                if args.is_empty() {
                                    log_format(&mut write, &username, "ERR 400 MISSING_ITEM_NAME").await.expect("Can't send missing item name error");
                                    continue;
                                }
                                let result = take(username.clone(), args.to_string(), Arc::clone(&state)).await;
                                if let Ok(response) = result {
                                    log_format(&mut write, &username, &response).await.expect("Can't send take response");
                                } else {
                                    log_format(&mut write, &username, "ERR 404 ITEM_NOT_FOUND").await.expect("Can't send take error response");
                                }
                            },
                            "DROP" => {
                                if args.is_empty() {
                                    log_format(&mut write, &username, "ERR 400 MISSING_ITEM_NAME").await.expect("Can't send missing item name error");
                                    continue;
                                }
                                let result = crate::items::drop(username.clone(), args.to_string(), Arc::clone(&state)).await;
                                if let Ok(response) = result {
                                    log_format(&mut write, &username, &response).await.expect("Can't send drop response");
                                } else {
                                    log_format(&mut write, &username, "ERR 404 ITEM_NOT_IN_INVENTORY").await.expect("Can't send drop error response");
                                }
                            },
                            "INVENTORY" => {
                                let players = state.players.lock().await;
                                let player = players.get(&username).unwrap();
                                let response = format!("OK {}", serde_json::to_string(&player.inventory).unwrap());
                                drop(players);
                                log_format(&mut write, &username, &response).await.expect("Can't send inventory response");
                            },
                            "QUEST" => {
                                if args.is_empty() {
                                    log_format(&mut write, &username, "ERR 400 MISSING_NPC_NAME").await.expect("Can't send missing NPC name error");
                                    continue;
                                }
                                let res = quest(username.clone(), args, Arc::clone(&state)).await;
                                log_format(&mut write, &username, &res).await.expect("Can't send quest response");
                            },
                            "QUESTS" => {
                                let res = get_quests(username.clone(), Arc::clone(&state)).await;
                                log_format(&mut write, &username, &res).await.expect("Can't send quests response");
                            }
                            "ATTACK" => {
                                if args.is_empty() {
                                    log_format(&mut write, &username, "ERR 400 MISSING_NPC_NAME").await.expect("Can't send missing NPC name error");
                                    continue;
                                }
                                let res = attack(username.clone(), args, Arc::clone(&state)).await;
                                log_format(&mut write, &username, &res).await.expect("Can't send attack response");
                            },
                            "DEFEND" => {
                                let res = defend(username.clone(), Arc::clone(&state)).await;
                                log_format(&mut write, &username, &res).await.expect("Can't send defend response");
                            },
                            "FLEE" => {
                                let res = flee(username.clone(), Arc::clone(&state)).await;
                                log_format(&mut write, &username, &res).await.expect("Can't send flee response");
                            },
                            "USE_ITEM" | "USE" => {
                                if args.is_empty() {
                                    log_format(&mut write, &username, "ERR 400 MISSING_ITEM_NAME").await.expect("Can't send missing item name error");
                                    continue;
                                }
                                let res = use_item(username.clone(), args, Arc::clone(&state)).await;
                                log_format(&mut write, &username, &res).await.expect("Can't send use response");
                            },
                            "STATUS" => {
                                let res = status(username.clone(), Arc::clone(&state)).await;
                                log_format(&mut write, &username, &res).await.expect("Can't send status response");
                            },
                            "GROUP" => {
                                let arg = args.split(' ').next().unwrap_or("");
                                match arg {
                                    "CREATE" => {
                                        match group_create(username.clone().as_str(), Arc::clone(&state)).await {
                                            Ok(group_id) => {
                                                let response = format!("OK group={}", group_id);
                                                log_format(&mut write, &username, &response).await.expect("Can't send group created response");
                                            },
                                            Err(e) => {
                                                let response = group_err_line(&e);
                                                log_format(&mut write, &username, &response).await.expect("Can't send group creation error response");
                                            }
                                        }
                                    },
                                    "INVITE" => {
                                        let players = state.players.lock().await;
                                        let player_name = args.strip_prefix("INVITE ").unwrap_or("").trim();
                                        let group_id = players.get(&username).and_then(|player| player.group.clone());
                                        let Some(group_id) = group_id else {
                                            drop(players);
                                            log_format(&mut write, &username, "ERR 401 NOT_IN_GROUP").await.expect("Can't send not in group error");
                                            continue;
                                        };
                                        drop(players);
                                        match group_invite(group_id.as_str(), player_name, Arc::clone(&state)).await {
                                            Ok(_) => {
                                                log_format(&mut write, &username, "OK").await.expect("Can't send group invite response");
                                            },
                                            Err(e) => {
                                                let response = group_err_line(&e);
                                                log_format(&mut write, &username, &response).await.expect("Can't send group invite error response");
                                            }
                                        }
                                    },
                                    "JOIN" => {
                                        let leader_name = args.strip_prefix("JOIN ").unwrap_or("").trim();
                                        match group_join(leader_name, username.clone().as_str(), Arc::clone(&state)).await {
                                            Ok(group_id) => {
                                                let response = format!("OK group={}", group_id);
                                                log_format(&mut write, &username, &response).await.expect("Can't send group join response");
                                            },
                                            Err(e) => {
                                                let response = group_err_line(&e);
                                                log_format(&mut write, &username, &response).await.expect("Can't send group join error response");
                                            }
                                        }
                                    },
                                    "LEAVE" => {
                                        match group_leave(username.clone().as_str(), Arc::clone(&state)).await {
                                            Ok(_) => {
                                                log_format(&mut write, &username, "OK").await.expect("Can't send group leave response");
                                            },
                                            Err(e) => {
                                                let response = group_err_line(&e);
                                                log_format(&mut write, &username, &response).await.expect("Can't send group leave error response");
                                            }
                                        }
                                    },
                                    _ => {
                                        log_format(&mut write, &username, "ERR 400 UNKNOWN_GROUP_COMMAND").await.expect("Can't send unknown group command error");
                                    }
                                }
                            },
                            _ => {
                                log_format(&mut write, &username, "ERR 400 UNKNOWN_COMMAND").await.expect("Can't send unknown command error");
                            }
                        }
                    },
                    Ok(None) => {
                        reason = "eof";
                        break;
                    },
                    Err(e) => {
                        logs_format::log_output("ERROR", "READ_ERROR", serde_json::json!({
                            "player": username, "reason": e.to_string()
                        }));
                        reason = "error";
                        break;
                    }
                }
            }
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    log_format(&mut write, &username, &msg).await.expect("Can't send message");
                }
            }
        }
    }
    logs_format::log_output("INFO", "DISCONNECT", serde_json::json!({
        "player": username, "reason": reason, "ip": ip
    }));
    remove_player(&username, Arc::clone(&state)).await;
	get_stats(&state).await;
}

pub async fn handle_client(socket: TcpStream, state: Arc<SharedState>) {
    let ip = socket
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();
    log_format(&mut writer, UNAUTHENTICATED, "OK hello proto=1")
        .await
        .expect("Can't send greeting message");
    let (username, rx) = loop {
        match handle_client_auth(&mut lines, &mut writer, Arc::clone(&state)).await {
            Ok((username, rx)) => break (username, rx),
            Err(e) => {
                let response = format!("ERR {}", e);
                if log_format(&mut writer, UNAUTHENTICATED, &response).await.is_err() {
                    return;
                }
            }
        }
    };
    handle_commands(lines, writer, rx, username, ip, Arc::clone(&state)).await;
}

async fn handle_client_auth(
    lines: &mut Lines<BufReader<OwnedReadHalf>>,
    writer: &mut OwnedWriteHalf,
    state: Arc<SharedState>,
) -> Result<(String, UnboundedReceiver<String>), UserError> {
    let line = lines
        .next_line()
        .await
        .map_err(|_| UserError::InvalidRead)?;
    let username = verify_authentication(line, Arc::clone(&state)).await?;
    let (tx, rx) = mpsc::unbounded_channel();
    add_player(username.clone(), Arc::clone(&state), tx)
        .await
        .map_err(|_| UserError::Internal)?;
    log_format(writer, &username, "OK connected")
        .await
        .map_err(|_| UserError::Internal)?;
	get_stats(&state).await;
    Ok((username, rx))
}
