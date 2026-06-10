use crate::chat::chat_room;
use crate::look::look;
use crate::move_cmd::move_cmd;
use crate::state::Player;
use crate::state::SharedState;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug, Error)]
enum UserError {
    #[error("INVALID_USERNAME")]
    InvalidUsername,
    #[error("ALREADY_EXIST")]
    AlreadyExist,
    #[error("BAD_PREFIX")]
    BadPrefix,
    #[error("INVALID_READ")]
    InvalidRead,
}

async fn verify_authentication(
    line: Option<String>,
    state: Arc<SharedState>,
) -> Result<String, UserError> {
    let line = line.ok_or(UserError::InvalidRead)?;
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
    //println!("{:#?}", initial_room_state);
    Ok(())
}

async fn remove_player(username: &str, state: Arc<SharedState>) {
    let mut world_state = state.world_state.lock().await;
    let mut players = state.players.lock().await;
    let player = players.get(username).unwrap();
    let room = world_state.room.get_mut(player.room.as_str()).unwrap();
    room.players.retain(|p| p != username);
    players.remove(username);
}

async fn handle_commands(
    mut lines: Lines<BufReader<OwnedReadHalf>>,
    mut write: OwnedWriteHalf,
    mut rx: mpsc::UnboundedReceiver<String>,
    username: String,
    state: Arc<SharedState>,
) {
    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        let mut parts = line.splitn(2, ' ');
                        let command = parts.next().unwrap_or("");
                        let args = parts.next().unwrap_or("").trim();
                        match command {
                            "LOOK" => {
                                write.write_all(format!("OK {}\n", look(username.clone(), Arc::clone(&state)).await).as_bytes()).await.expect("Can't send look response");
                            },
                            "MOVE" => {
                                match move_cmd(username.clone(), args.to_string(), Arc::clone(&state)).await {
                                    Ok(new_room) => {
                                        write.write_all(format!("OK {}\n", new_room).as_bytes()).await.expect("Can't send move response");
                                    },
                                    Err(e) => {
                                        write.write_all(format!("ERR {}\n", e).as_bytes()).await.expect("Can't send move error response");
                                    }
                                }
                            },
                            "QUIT" => {
                                write.write_all(b"OK bye\n").await.expect("Can't send goodbye message");
                                break;
                            },
                            "CHAT" => {
                                let scope = args.splitn(2, ' ').next().unwrap_or("");
                                //println!("Chat command with scope: {}", scope);
                                match scope {
                                    "ROOM" => {
                                        let message = args.strip_prefix("ROOM ").unwrap_or("").trim();
                                        chat_room(message.to_string(), username.clone(), Arc::clone(&state)).await;
                                    },
                                    _ => {
                                        write.write_all(b"ERR UNKNOWN_SCOPE\n").await.expect("Can't send unknown scope error");
                                    }
                                }
                            },
                            "WHO" => {
                                write.write_all(crate::who::who(username.clone(), Arc::clone(&state)).await.as_bytes()).await.expect("Can't send who response");
                            },
                            _ => {
                                println!("Unknown command from {}: {}", username, command);
                            }
                        }
                    },
                    Ok(None) => {
                        println!("Client {} disconnected", username);
                        break;
                    },
                    Err(e) => {
                        println!("Error reading from {}: {}", username, e);
                        break;
                    }
                }
            }
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    write.write_all(format!("{}\n", msg).as_bytes()).await.expect("Can't send message");
                }
            }
        }
    }
    remove_player(&username, Arc::clone(&state)).await;
}

async fn print_debug_state(state: Arc<SharedState>) {
    let world_state = state.world_state.lock().await;
    println!("{:#?}", world_state);
}

pub async fn handle_client(socket: TcpStream, state: Arc<SharedState>) {
    println!("New client connected. Need to authenticate");
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();
    writer
        .write_all(b"OK hello proto=1\n")
        .await
        .expect("Can't send great message");
    let (username, rx) = loop {
        match handle_client_auth(&mut lines, &mut writer, Arc::clone(&state)).await {
            Ok((username, rx)) => break (username, rx),
            Err(e) => {
                if writer
                    .write_all(format!("ERR {}\n", e).as_bytes())
                    .await
                    .is_err()
                {
                    return;
                }
            }
        }
    };
    handle_commands(lines, writer, rx, username, Arc::clone(&state)).await;
}

pub async fn handle_client_auth(
    lines: &mut Lines<BufReader<OwnedReadHalf>>,
    writer: &mut OwnedWriteHalf,
    state: Arc<SharedState>,
) -> Result<(String, UnboundedReceiver<String>), Box<dyn std::error::Error + Send + Sync>> {
    let line = lines.next_line().await?;
    let username = verify_authentication(line, Arc::clone(&state)).await?;
    let (tx, rx) = mpsc::unbounded_channel();
    add_player(username.clone(), Arc::clone(&state), tx).await?;
    writer.write_all(b"OK connected\n").await?;
    println!("Client authenticated");
    Ok((username, rx))
}
