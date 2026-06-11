use crate::parser::ServerMessage;

pub fn auth(
    rx_incoming: &std::sync::mpsc::Receiver<ServerMessage>,
    tx_outgoing: &std::sync::mpsc::Sender<String>,
    username: String,
) -> Result<(), String> {
    loop {
        let msg = rx_incoming
            .recv()
            .map_err(|e| format!("Failed to receive: {}", e))?;
        match msg {
            ServerMessage::Ok(data) => {
                tx_outgoing
                    .send(format!("CONNECT {}", username))
                    .map_err(|e| format!("Failed to send: {}", e))?;
                let msg = rx_incoming
                    .recv()
                    .map_err(|e| format!("Failed to receive: {}", e))?;

                match msg {
                    ServerMessage::Ok(data) => break,
                    ServerMessage::Err { code, message } => return Err(format!("ERR {}: {}", code, message)),
                    ServerMessage::Evt { evt_type, data } => return Err(format!("Error: {}", data)),
                }
            }
            ServerMessage::Err { code, message } => return Err(format!("ERR {}: {}", code, message)),
            ServerMessage::Evt { evt_type, data } => return Err(format!("Error: {}", data)),
        }
    }
    Ok(())
}
