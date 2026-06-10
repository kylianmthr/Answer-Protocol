use std::env;
mod state;
use state::SharedState;
mod server;
use server::run_server;
mod broadcast;
mod client;
mod look;
mod move_cmd;
mod validate;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <number> <map_path>", args[0]);
        std::process::exit(1);
    }
    let port: u16 = args[1].parse().expect("Please provide a valid number");
    let path = args[2].clone();
    println!("Starting server on port {}", port);
    run_server(port, SharedState::new(path.to_string())).await;
}
