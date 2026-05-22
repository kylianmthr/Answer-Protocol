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

impl Player {
    pub fn new(username: &str, tx: mpsc::UnboundedSender<String>) -> Self {
        // Test si le username est trop court etc..
        Self {
            name: username.to_string(),
            room: "Initial room".to_string(),
            hp: 100,
            inventory: Vec::new(),
            tx: tx,
        }
    }
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
