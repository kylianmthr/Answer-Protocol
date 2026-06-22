use std::sync::Arc;

use crate::state::SharedState;

pub async fn talk(npc_name_or_id: &str, shared_state: Arc<SharedState>) -> Result<String, String> {
    let world_data = shared_state.world_data.lock().await;
    let mut npc = world_data.world.npcs.get(npc_name_or_id);
    if npc.is_none() {
        npc = world_data
            .world
            .npcs
            .values()
            .find(|npc| npc.name == npc_name_or_id);
    }
    npc.ok_or_else(|| format!("NPC '{}' not found", npc_name_or_id))
        .map(|npc| serde_json::to_string(&npc).unwrap())
}
