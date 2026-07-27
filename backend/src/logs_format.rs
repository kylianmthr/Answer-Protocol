use serde_json::{self, json};

pub fn time_format() -> String {
	chrono::Utc::now().to_rfc3339()
}


pub fn log_output(level: &str,event: &str,fields: serde_json::Value) {
	let json_format = json!({
		"timestamp": time_format(),
		"level": level,
		"event": event,
		"data": fields,
	});
	println!("{}", json_format)
}