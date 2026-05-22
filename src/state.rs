use tokio::sync::{Mutex, mpsc};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Player {
    name: String,
    room: String,
    hp: i32,
    inventory: Vec<String>,
    tx: mpsc::UnboundedSender<String>,
}

struct World {

}

pub struct SharedState {
    pub players: Mutex<HashMap<String, Player>>,
    world: Mutex<World>,
}

impl SharedState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            players: Mutex::new(HashMap::new()),
            world: Mutex::new(World {

            })
        })
    }
}
