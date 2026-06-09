use crate::parser::ServerMessage;

pub fn auth(
    rx_incoming: std::sync::mpsc::Receiver<ServerMessage>,
    tx_outgoing: std::sync::mpsc::Sender<String>,
) -> (
    std::sync::mpsc::Receiver<ServerMessage>,
    std::sync::mpsc::Sender<String>,
) {
    loop {
        let msg = rx_incoming
            .recv()
            .expect("Failed to receive message from reader thread");
        match msg {
            ServerMessage::Ok(data) => {
                let username = "test";
                tx_outgoing
                    .send(format!("CONNECT {}", username))
                    .expect("Failed to send message");
                let msg = rx_incoming
                    .recv()
                    .expect("Failed to receive message from reader thread");

                match msg {
                    ServerMessage::Ok(data) => break,
                    ServerMessage::Err { code, message } => eprintln!("ERR {}: {}", code, message),
                    ServerMessage::Evt { evt_type, data } => eprintln!("Error: {}", data),
                }
            }
            ServerMessage::Err { code, message } => eprintln!("ERR {}: {}", code, message),
            ServerMessage::Evt { evt_type, data } => eprintln!("Error: {}", data),
        }
    }
    (rx_incoming, tx_outgoing)
}
