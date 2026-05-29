use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

#[derive(Clone)]
pub struct Player {
    name: String,
    hp: i32,
    inventory: Vec<String>,
    tx: mpsc::UnboundedSender<String>,
    pub room: String,
}

impl Player {
    pub fn new(username: &str, room: &str, tx: mpsc::UnboundedSender<String>) -> Self {
        // Test si le username est trop court etc..
        Self {
            name: username.to_string(),
            hp: 100,
            inventory: Vec::new(),
            tx,
            room: room.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Room {
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, String>,
    pub items: Vec<String>,
    pub npcs: Vec<String>,
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
    room: String,
}

#[derive(Debug, Deserialize)]
pub struct World {
    pub initial_room: String,
    pub rooms: HashMap<String, Room>,
    items: HashMap<String, Item>,
    npcs: HashMap<String, Npc>,
}

#[derive(Debug, Deserialize)]
pub struct WorldData {
    pub world: World,
}

impl WorldData {
    pub fn load_from_file(path: &str) -> Self {
        let content = std::fs::read_to_string(path).expect("Could not read world data file");
        let world: Self = serde_yaml::from_str(&content).expect("Could not parse world data");
        return world;
    }
}

#[derive(Debug)]
pub struct NpcState {
    room: String,
    hp: i32,
}

#[derive(Debug, Serialize)]
pub struct RoomState {
    items: Vec<String>,
    npcs: Vec<String>,
    pub players: Vec<String>,
}

#[derive(Debug)]
pub struct WorldState {
    pub room: HashMap<String, RoomState>,
    npcs: HashMap<String, NpcState>,
}

pub struct SharedState {
    pub players: Mutex<HashMap<String, Player>>,
    pub world_data: Mutex<WorldData>,
    pub world_state: Mutex<WorldState>,
}

impl WorldState {
    pub fn from_world_data(world_data: &WorldData) -> Self {
        let mut room_state = HashMap::new();
        for (room_name, room) in &world_data.world.rooms {
            room_state.insert(
                room_name.clone(),
                RoomState {
                    items: room.items.clone(),
                    npcs: room.npcs.clone(),
                    players: Vec::new(),
                },
            );
        }
        let mut npc_state = HashMap::new();
        for (npc_name, npc) in &world_data.world.npcs {
            npc_state.insert(
                npc_name.clone(),
                NpcState {
                    room: npc.room.clone(),
                    hp: npc.hp,
                },
            );
        }
        Self {
            room: room_state,
            npcs: npc_state,
        }
    }
}

impl SharedState {
    pub fn new(path: String) -> Arc<Self> {
        Arc::new(Self {
            players: Mutex::new(HashMap::new()),
            world_data: Mutex::new(WorldData::load_from_file(&path)),
            world_state: Mutex::new(WorldState::from_world_data(&WorldData::load_from_file(
                &path,
            ))),
        })
    }
}
