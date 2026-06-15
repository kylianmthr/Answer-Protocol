mod app;
mod game_mod {
	pub mod state_mod;
	pub mod room_1;
	pub mod room_2;
	pub mod room_3;
	pub mod room_4;
}

mod action_game;
use std::io::{BufRead, BufReader, Write};
use parser::{ServerMessage, parser};
use std::net::TcpStream;
use std::sync::mpsc;
use app::MyTap;
use std::thread;
use std::env;
mod parser;
mod auth;

use crate::app::font_style;

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <number>", args[0]);
        std::process::exit(1);
    }
    let port: u16 = args[1].parse().expect("Please provide a valid number");
    let stream =
        TcpStream::connect(format!("127.0.0.1:{}", port)).expect("Could not connect to server");
    let stream_read = stream.try_clone().expect("Failed to clone stream");
    let mut stream_write = stream.try_clone().expect("Failed to clone stream");
    let (tx_incoming, rx_incoming) = mpsc::channel::<ServerMessage>();
    let (tx_outgoing, rx_outgoing) = mpsc::channel::<String>();
    thread::spawn(move || {
        let cmd = BufReader::new(stream_read);
        let line = cmd.lines();
        line.for_each(|line| match line {
            Ok(line) => match parser(&line) {
                Ok(msg) => tx_incoming
                    .send(msg)
                    .expect("Failed to send message to main thread"),
                Err(e) => eprintln!("Failed to parse message: {}", e),
            },
            Err(e) => eprintln!("Failed to read line: {}", e),
        });
    });
    thread::spawn(move || {
        for msg in rx_outgoing {
            let msg = format!("{}\n", msg);
            if let Err(e) = stream_write.write_all(msg.as_bytes()) {
                eprintln!("Failed to send message: {}", e);
                break;
            }
        }
    });
    println!("Listening on port {}", port);
    rx_incoming
        .recv()
        .expect("Failed to receive message from server");
    let options_visualizeur = eframe::NativeOptions::default();
    eframe::run_native(
        "Answer Protocol",
        options_visualizeur,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
			font_style(&cc.egui_ctx);
            Ok(Box::new(MyTap::new(rx_incoming, tx_outgoing)))
        }),
    )
}
