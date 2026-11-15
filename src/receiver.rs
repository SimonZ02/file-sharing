use std::path::Path;

use iroh::{
    Endpoint,
    endpoint::{Connection, RecvStream, SendStream},
    protocol::{AcceptError, ProtocolHandler, Router},
};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::{
    ALPN, protocol::{Message, read_message, send_message}, sender::send_file,
};

/*
Receiver: open to receive files.
- starts first and waits to be discovered by senders (like AirDrop)
*/

const MAX_FILE_SIZE: usize = 10_000_000; // 10mb

const TEST_DIR: &str = "test_downloads"; // ist temporär, werden wir dann mit dem UI auswählen denke ich (falls wir das wollen)

pub async fn listen(endpoint: Endpoint) -> anyhow::Result<()> {
    println!("my id: {}", endpoint.id());

    let router = Router::builder(endpoint).accept(ALPN, FileHandler).spawn();
    println!("waiting for senders... (ctrl c to stop)");

    tokio::signal::ctrl_c().await?;
    router.shutdown().await?;
    Ok(())
}

#[derive(Debug, Clone)]
struct FileHandler;

impl ProtocolHandler for FileHandler {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        println!("a sender connected!");

        while let Ok((send, recv)) = connection.accept_bi().await {
            if let Err(e) = handle_transfer(send, recv).await {
                println!("transfer failed: {e}");
            }
        }

        println!("the sender left");
        Ok(())
    }
}

async fn handle_transfer(mut send: SendStream, mut recv: RecvStream) -> anyhow::Result<()> {
    let (file_name, file_size) = read_offer(&mut recv).await?;

    let accepted = ask_user(&file_name, file_size).await;  

    let message = Message::Answer { accepted };
    send_message(&mut send, &message).await?;


    if !accepted {
        send.finish()?;

        println!("declined by user");
        return Ok(());
    }

 
    let bytes = match recv.read_to_end(MAX_FILE_SIZE).await {
        Ok(bytes) => bytes,
        Err(e) => {
            anyhow::bail!("could not read file: {e}");
        }
    };

    save_file(&file_name, &bytes).await?;

    let reply = format!("saved {} bytes", bytes.len());
    if let Err(e) = send.write_all(reply.as_bytes()).await {
        anyhow::bail!("could not reply: {e}");
    }
    send.finish()?;

    Ok(())
}

async fn read_offer(recv: &mut RecvStream) -> anyhow::Result<(String, u64)> {
    let offer = match read_message(recv).await {
        Ok(offer) => offer,
        Err(e) => {
            anyhow::bail!("could not read message: {e}");
        }
    };
    println!("{:?}", offer);

    let (file_name_raw, file_size) = match offer {
        Message::Offer { file_name, size } => (file_name, size),
        _ => {
            anyhow::bail!("offer not correct");
        }
    };
    println!("this is the file_name: {}", file_name_raw);

    let file_path = Path::new(&file_name_raw);

    let file_name = match file_path.file_name() {
        Some(file_name) => file_name,
        None => {
            anyhow::bail!("could not resolve file_name from given path");
        }
    };

    Ok((file_name.to_string_lossy().to_string(), file_size))
}

async fn ask_user(file_name: &str, file_size: u64) -> bool {
    println!("received an offer: file_name = {} and size = {}", file_name, file_size);
    println!("Do you want to receive this file [y/n]?");

    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    let answer = match stdin.next_line().await {
        Ok(Some(answer)) => answer,
        Ok(None) => String::new(),
        Err(e) => {
            println!("could not read input: {e}");
            String::new()
        }
    };

    let accepted = match answer.trim().to_lowercase().as_str() {
        "yes" | "y" | "ye" | "ys" => true,
        _ => false,
    };

    accepted
}

async fn save_file(file_name: &str, bytes: &[u8]) -> anyhow::Result<()> {
    if let Err(e) = tokio::fs::create_dir_all(TEST_DIR).await {
        anyhow::bail!("could not create dir: {}", e);
    }

    let final_path = Path::new(TEST_DIR).join(file_name);

    if let Err(e) = tokio::fs::write(final_path, bytes).await {
        anyhow::bail!("could not save file: {e}");
    }
    println!("saved {} bytes as {}", bytes.len(), file_name);

    Ok(())
}
