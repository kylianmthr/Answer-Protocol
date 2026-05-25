use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

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

#[derive(Debug, Deserialize)]
pub struct Room {
    name: String,
    description: String,
    exits: HashMap<String, String>,
    items: Vec<String>,
    npcs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    name: String,
    description: String,
    obtainable: bool,
}

#[derive(Debug, Deserialize)]
pub struct Npc {
    name: String,
    description: String,
    dialogue: Vec<String>,
    hp: i32,
    hostile: bool,
}

#[derive(Debug, Deserialize)]
struct World {
    rooms: HashMap<String, Room>,
    items: HashMap<String, Item>,
    npcs: HashMap<String, Npc>,
}

#[derive(Debug, Deserialize)]
pub struct WorldData {
    world: World,
}

impl WorldData {
    pub fn load_from_file(path: &str) -> Self {
        let content = std::fs::read_to_string(path).expect("Could not read world data file");
        let world: Self = serde_yaml::from_str(&content).expect("Could not parse world data");
        println!("World data loaded successfully: {:#?}", world);
        return world;
    }
}

pub struct SharedState {
    pub players: Mutex<HashMap<String, Player>>,
    world: Mutex<WorldData>,
}

impl SharedState {
    pub fn new(path: String) -> Arc<Self> {
        Arc::new(Self {
            players: Mutex::new(HashMap::new()),
            world: Mutex::new(WorldData::load_from_file(&path)),
        })
    }
}
