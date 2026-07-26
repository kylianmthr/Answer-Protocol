use crate::client::handle_client;
use crate::state::SharedState;
use crate::logs_format::log_output;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::flood_systeme::check_flood;

pub async fn run_server(port: u16, state: Arc<SharedState>) {
    let host = "127.0.0.1";
    let address = format!("{host}:{port}");
    let listener = TcpListener::bind(address)
        .await
        .expect("Could not bind to address");
    loop {
        let (socket, addr) = listener
            .accept()
            .await
            .expect("Could not accept connection");
		log_output("INFO", "IP", serde_json::json!({
							"CONNECT_ID=": addr
						}));
		check_flood(&addr.ip().to_string(), &state).await;
        let state_clone = Arc::clone(&state);
        tokio::spawn(async move {
            handle_client(socket, state_clone).await;
        });
    }
}
