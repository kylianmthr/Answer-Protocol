use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::logs_format::log_output;
use crate::state::{SharedState};


const CMD_WINDOW: Duration = Duration::from_secs(5);
const MAX_CMD: usize = 10;

const CONNECTIONS: Duration = Duration::from_secs(5);
const MAX_CONNECTIONS: usize = 5;

pub async fn command_check_flooding(username: &str, state: &Arc<SharedState>) {
	let mut abusing = state.abuse.lock().await;
	let now = Instant::now();
	let entry_into =  abusing.commands.entry(username.to_string()).or_insert_with(VecDeque::new);
    entry_into.push_back(now);
    while matches!(entry_into.front(), Some(front) if now.duration_since(*front) > CMD_WINDOW) {
        entry_into.pop_front();
    }
    if entry_into.len() > MAX_CMD {
        log_output("WARN", "COMMAND_FLOOD", serde_json::json!({
            "player": username, "count": entry_into.len(), "window_s": CMD_WINDOW.as_secs()
        }));
    }
}

pub async fn check_flood(ip: &str, state: &Arc<SharedState>) {
	 let mut abuse = state.abuse.lock().await;
    let now = Instant::now();
    let entry = abuse.connected.entry(ip.to_string()).or_insert_with(VecDeque::new);
    entry.push_back(now);
    while matches!(entry.front(), Some(front) if now.duration_since(*front) > CONNECTIONS) {
        entry.pop_front();
    }
    if entry.len() > MAX_CONNECTIONS {
        log_output("WARN", "RAPID_CONNECT", serde_json::json!({
            "ip": ip, "count": entry.len(), "window_s": CONNECTIONS.as_secs()
        }));
    }
}
