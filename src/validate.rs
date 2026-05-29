use std::collections::HashMap;

use validator::ValidationError;

pub fn validate_exits(exits: &HashMap<String, String>) -> Result<(), ValidationError> {
    let valid_directions = ["north", "south", "east", "west"];

    for key in exits.keys() {
        if !valid_directions.contains(&key.as_str()) {
            return Err(ValidationError::new("invalid_exit_direction"));
        }
    }
    Ok(())
}
