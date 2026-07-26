use std::sync::Arc;
use crate::logs_format::log_output;

use crate::state::{PlayerQuest, Quest, QuestStatus, SharedState};

fn quest_json(quest: &Quest, status: &str) -> String {
    serde_json::json!({
        "quest_id": quest.id,
        "name": quest.name,
        "description": quest.description,
        "reward": quest.reward,
        "status": status,
    })
    .to_string()
}

pub async fn quest(username: String, npc_name_or_id: &str, state: Arc<SharedState>) -> String {
    let world_data = state.world_data.lock().await;
    let npc_data = world_data.world.npcs.get(npc_name_or_id).or_else(|| {
        world_data
            .world
            .npcs
            .values()
            .find(|npc| npc.name == npc_name_or_id)
    });
    let npc = match npc_data {
        Some(npc) => npc,
        None => return "ERR 404 NPC_NOT_FOUND".to_string(),
    };
    let quest = match &npc.quest {
        Some(quest) => quest,
        None => return "ERR 406 NO_QUEST_AVAILABLE".to_string(),
    };
    let mut players = state.players.lock().await;
    let player = players.get_mut(&username).unwrap();
    match player.quests.iter().position(|pq| pq.quest.id == quest.id) {
        None => {
            player.quests.push(PlayerQuest {
                quest: quest.clone(),
                status: QuestStatus::Active,
            });
            format!("OK {}", quest_json(quest, "active"))
        }
        Some(i) if player.quests[i].status == QuestStatus::Completed => {
            "ERR 406 NO_QUEST_AVAILABLE".to_string()
        }
        Some(i) => match player
            .inventory
            .iter()
            .position(|item| item == &quest.objective)
        {
            Some(item_pos) => {
                player.inventory.remove(item_pos);
                player.inventory.push(quest.reward.clone());
                player.quests[i].status = QuestStatus::Completed;
                format!("OK {}", quest_json(quest, "completed"))
            }
            None => format!("OK {}", quest_json(quest, "active")),
        },
    }
}

pub async fn get_quests(username: String, state: Arc<SharedState>) -> String {
    let players = state.players.lock().await;
    let player = players.get(&username).unwrap();
    let quests: Vec<serde_json::Value> = player
        .quests
        .iter()
        .map(|pq| {
            let objective_done = pq.status == QuestStatus::Completed
                || player.inventory.contains(&pq.quest.objective);
            serde_json::json!({
                "quest_id": pq.quest.id,
                "status": pq.status.as_str(),
                "progress": if objective_done { "1/1" } else { "0/1" },
            })
        })
        .collect();
    format!("OK {}", serde_json::to_string(&quests).unwrap())
}
