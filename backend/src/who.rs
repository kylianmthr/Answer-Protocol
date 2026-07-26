use std::sync::Arc;

use crate::state::SharedState;

pub async fn who(_username: String, state: Arc<SharedState>) -> String {
    let players = state.players.lock().await;
    format!("OK players={}", players.len())
}
