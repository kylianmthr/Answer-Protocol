use tokio::net::TcpStream;
use crate::state::SharedState;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};

async fn verify_authentication(line: String, state: Arc<SharedState>) -> bool {
    if let Some(username) = line.strip_prefix("CONNECT ") {
        if username.trim().len() == 0 {
            return false
        }
        let players = state.players.lock().await;
        if players.get(username).is_some() {
            return false
        }
        return true
    }
    return false
}

pub async fn handle_client(socket: TcpStream, state: Arc<SharedState>)
{
    println!("New client connected. Need to authenticate");
    let (reader, mut writer) = socket.into_split();
    writer.write_all(b"OK hello proto=1\n").await.expect("Can't send great message");
    let mut lines = BufReader::new(reader).lines();
    let mut authenticated = false;
    while !authenticated {
        let line = lines.next_line().await.expect("Can't read the line");
        match line {
            Some(l) => {
                authenticated = verify_authentication(l, Arc::clone(&state)).await;
                if !authenticated {
                    writer.write_all(b"ERR 500 Can't authenticated the user\n").await.expect("Can't send great message");
                }
            },
            None => return,
        }
    }
    writer.write_all(b"OK connected\n").await.expect("Can't send great message");
    println!("Client authenticated");
}
