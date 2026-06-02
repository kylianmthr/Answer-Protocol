use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use thiserror::Error;

enum EventType {
    PresenceEnter,
    PresenceLeave,
    RoomChat,
    GroupChat,
    GlobalChat,
    Invite,
    Join,
    Leave,
    Stats,
}

enum ServerMessage {
    Ok(String),
    Err { code: u32, message: String },
    Evt { evt_type: EventType, data: String },
}

#[derive(Debug, Error)]
enum UserError {
    #[error("INVALID_USERNAME")]
    InvalidUsername,
    #[error("ALREADY_EXIST")]
    AlreadyExist,
    #[error("BAD_PREFIX")]
    BadPrefix,
    #[error("INVALID_READ")]
    InvalidRead,
}

fn parser(msg: &str) -> Result<ServerMessage, &str> {
    let mut parts = msg.splitn(2, ' ');
    let status = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("").trim();
    match status {
        "OK" => Ok(ServerMessage::Ok(args.to_string())),
        "ERR" => {
            let mut splited_args = args.splitn(2, ' ');
            let code: u32 = splited_args
                .next()
                .unwrap_or("")
                .to_string()
                .parse()
                .unwrap_or(0);
            Ok(ServerMessage::Err {
                code: (code),
                message: (splited_args.next().unwrap_or("").to_string()),
            })
        }
        "EVT" => {
            let words: Vec<&str> = args.split_whitespace().collect();
            match words.as_slice() {
                ["ROOM", "PRESENCE", "ENTER", rest @ ..] => Ok(ServerMessage::Evt {
                    evt_type: EventType::PresenceEnter,
                    data: rest.join(" "),
                }),
                ["ROOM", "PRESENCE", "LEAVE", rest @ ..] => Ok(ServerMessage::Evt {
                    evt_type: EventType::PresenceLeave,
                    data: rest.join(" "),
                }),
                ["ROOM", "CHAT", rest @ ..] => Ok(ServerMessage::Evt {
                    evt_type: EventType::RoomChat,
                    data: rest.join(" "),
                }),
                ["GLOBAL", "CHAT", rest @ ..] => Ok(ServerMessage::Evt {
                    evt_type: EventType::GlobalChat,
                    data: rest.join(" "),
                }),
                ["GROUP", "CHAT", rest @ ..] => Ok(ServerMessage::Evt {
                    evt_type: EventType::GroupChat,
                    data: rest.join(" "),
                }),
                ["GROUP", "INVITE", rest @ ..] => Ok(ServerMessage::Evt {
                    evt_type: EventType::Invite,
                    data: rest.join(" "),
                }),
                ["GROUP", "JOIN", rest @ ..] => Ok(ServerMessage::Evt {
                    evt_type: EventType::Join,
                    data: rest.join(" "),
                }),
                ["GROUP", "LEAVE", rest @ ..] => Ok(ServerMessage::Evt {
                    evt_type: EventType::Leave,
                    data: rest.join(" "),
                }),
                ["STATS", rest @ ..] => Ok(ServerMessage::Evt {
                    evt_type: EventType::Stats,
                    data: rest.join(" "),
                }),
                _ => Err("jsp"),
            }
        }
        _ => Err("jsp"),
    }
}

fn main() {
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
}
