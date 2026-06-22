use crate::validate::validate_exits;
use crate::validate::validate_yaml;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use validator::Validate;
use validator::ValidationErrors;

#[derive(Clone, Validate)]
pub struct Player {
    #[validate(length(min = 3, max = 20))]
    pub name: String,
    pub hp: i32,
    pub inventory: Vec<String>,
    pub tx: mpsc::UnboundedSender<String>,
    pub room: String,
    pub invitations: Vec<Group>,
    pub group: Option<String>,
}

impl Player {
    pub fn new(
        username: &str,
        room: &str,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<Self, ValidationErrors> {
        let player = Self {
            name: username.to_string(),
            hp: 100,
            inventory: Vec::new(),
            tx,
            room: room.to_string(),
            invitations: Vec::new(),
            group: None,
        };
        player.validate()?;
        Ok(player)
    }
}

#[derive(Clone, Validate)]
pub struct Group {
    pub id: String,
    pub members: Vec<Player>,
}

impl Group {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, player: Player) {
        self.members.push(player);
    }

    pub fn remove_member(&mut self, player_name: &str) {
        self.members.retain(|member| member.name != player_name);
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct Room {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(custom(function = "validate_exits"))]
    pub exits: HashMap<String, String>,
    pub items: Vec<String>,
    pub npcs: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Item {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub description: String,
    pub obtainable: bool,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Npc {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub description: String,
    pub dialogue: Vec<String>,
    #[validate(range(min = 1, max = 100))]
    pub hp: i32,
    pub hostile: bool,
    #[validate(length(min = 1, max = 255))]
    pub room: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct World {
    pub initial_room: String,
    #[validate(nested)]
    pub rooms: HashMap<String, Room>,
    #[validate(nested)]
    pub items: HashMap<String, Item>,
    #[validate(nested)]
    pub npcs: HashMap<String, Npc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct WorldData {
    #[validate(nested)]
    pub world: World,
}

impl WorldData {
    pub fn load_from_file(path: &str) -> Self {
        let content = std::fs::read_to_string(path).expect("Could not read world data file");
        let world: Self = serde_yaml::from_str(&content).expect("Could not parse world data");
        world.validate().expect("World data validation failed");
        validate_yaml(&world).expect("World data cross-reference validation failed");
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
    pub id: String,
    pub items: Vec<String>,
    pub npcs: Vec<String>,
    pub players: Vec<String>,
    pub exits_rooms: Vec<String>,
}

#[derive(Debug)]
pub struct WorldState {
    pub room: HashMap<String, RoomState>,
    pub npcs: HashMap<String, NpcState>,
}

pub struct SharedState {
    pub players: Mutex<HashMap<String, Player>>,
    pub world_data: Mutex<WorldData>,
    pub world_state: Mutex<WorldState>,
    pub groups: Mutex<HashMap<String, Group>>,
}

impl WorldState {
    pub fn from_world_data(world_data: &WorldData) -> Self {
        let mut room_state = HashMap::new();
        for (room_name, room) in &world_data.world.rooms {
            room_state.insert(
                room_name.clone(),
                RoomState {
                    id: room_name.clone(),
                    items: room.items.clone(),
                    npcs: room.npcs.clone(),
                    players: Vec::new(),
                    exits_rooms: room.exits.keys().cloned().collect(),
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
            groups: Mutex::new(HashMap::new()),
        })
    }
}
