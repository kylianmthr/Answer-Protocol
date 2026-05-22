use std::env;
mod state;
use state::SharedState;
mod server;
use server::run_server;
mod client;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <number>", args[0]);
        std::process::exit(1);
    }
    let port: u16 = args[1].parse().expect("Please provide a valid number");
    println!("Starting server on port {}", port);
    run_server(port, SharedState::new()).await;
}
