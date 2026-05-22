use tokio::net::TcpStream;
use crate::state::SharedState;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt, Lines};
use crate::state::Player;
use tokio::sync::mpsc;
use thiserror::Error;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;

#[derive(Debug, Error)]
enum UserError {
    #[error("INVALID_USERNAME")]
    InvalidUsername,
    #[error("ALREADY_EXIST")]
    AlreadyExist,
    #[error("BAD_PREFIX")]
    BadPrefix,
    #[error("INVALID_READ")]
    InvalidRead
}

async fn verify_authentication(line: Option<String>, state: Arc<SharedState>) -> Result<String, UserError> {
    let line = line.ok_or(UserError::InvalidRead)?;
    if let Some(username) = line.strip_prefix("CONNECT ") {
        if username.trim().is_empty() {
            return Err(UserError::InvalidUsername)
        }
        let players = state.players.lock().await;
        if players.get(username).is_some() {
            return Err(UserError::AlreadyExist)
        }
        return Ok(username.to_string())
    }
    Err(UserError::BadPrefix)
}

async fn add_player(username: String, state: Arc<SharedState>, tx: mpsc::UnboundedSender<String>) {
    let mut players = state.players.lock().await;
    let player = Player::new(&username, tx);
    players.insert(username, player);
}

pub async fn handle_client(socket: TcpStream, state: Arc<SharedState>) {
    println!("New client connected. Need to authenticate");
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();
    writer.write_all(b"OK hello proto=1\n").await.expect("Can't send great message");
    loop {
        match handle_client_auth(&mut lines, &mut writer, Arc::clone(&state)).await {
            Ok(_) => break,
            Err(e) => {
                if writer.write_all(format!("ERR {}\n", e).as_bytes()).await.is_err() {
                    return
                }
            }
        }
    }
}

pub async fn handle_client_auth(
    lines: &mut Lines<BufReader<OwnedReadHalf>>,
    writer: &mut OwnedWriteHalf,
    state: Arc<SharedState>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let line = lines.next_line().await?;
    let username = verify_authentication(line, Arc::clone(&state)).await?;
    let (tx, rx) = mpsc::unbounded_channel();
    add_player(username, Arc::clone(&state), tx);
    writer.write_all(b"OK connected\n").await?;
    println!("Client authenticated");
    Ok(())
}
