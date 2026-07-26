use chrono;
use serde_json;

pub fn time_format() -> String {
	chrono::Utc::now().to_rfc3339()
}


pub fn log_output(level: &str, message: &str) {
	println!("[{}] {} {}", time_format(), level, message);
}
