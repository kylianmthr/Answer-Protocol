use tokio::net::TcpStream;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Enter server adress: ");
    io::stdout().write_all(b"").await?; 
    io::stdout().flush().await?;

    let mut stdin_reader = BufReader::new(io::stdin());
    let mut addr = String::new();
    
    stdin_reader.read_line(&mut addr).await?;
    let addr = addr.trim();

    let stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Cannot connect to the server {} ({})", addr, e);
            return Ok(());
        }
    };
    println!("Connected to server: {}", addr);

    let (reader, mut writer) = stream.into_split();

    let read_task = tokio::spawn(async move {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        
        while buf_reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            print!("{}", line);
            line.clear();
        }
        println!("\nConnection closed");
    });

    let write_task = tokio::spawn(async move {
        let mut stdin = BufReader::new(io::stdin());
        let mut input = String::new();

        while stdin.read_line(&mut input).await.unwrap_or(0) > 0 {
            if writer.write_all(input.as_bytes()).await.is_err() {
                break;
            }
            input.clear();
        }
    });

    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }

    Ok(())
}