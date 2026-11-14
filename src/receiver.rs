use std::path::Path;

use iroh::{
    Endpoint,
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler, Router},
};

use crate::{ALPN, protocol::{read_message, Message}};

/*
Receiver: open to receive files.
- starts first and waits to be discovered by senders (like AirDrop)
*/

const MAX_FILE_SIZE: usize = 10_000_000; // 10mb

// const SAVE_PATH: &str = "received.bin";

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

        while let Ok((mut send, mut recv)) = connection.accept_bi().await {

            let offer = match read_message(&mut recv).await {
                Ok(offer) => offer,
                Err(e) => {
                    println!("could not read message: {e}");
                    continue;
                }
            };
            println!("{:?}", offer);

            let file_name_raw = match offer {
                Message::Offer {file_name, ..} => file_name,
                _ => {println!("offer not correct");
                continue;
                }
            };
            println!("this is the file_name: {}", file_name_raw);

            let file_path = Path::new(&file_name_raw);

            let file_name = match file_path.file_name() {
                Some(file_name) => file_name,
                None => { println!("could not resolve file_name from given path");
                            continue;
            }

            };



            let bytes = match recv.read_to_end(MAX_FILE_SIZE).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    println!("could not read file: {e}");
                    continue;
                }
            };


            if let Err(e) = tokio::fs::write(file_name, &bytes).await {
                println!("could not save file: {e}");
                continue;
            }
            println!("saved {} bytes as {}", bytes.len(),file_name.to_string_lossy());

            let reply = format!("saved {} bytes", bytes.len());
            if let Err(e) = send.write_all(reply.as_bytes()).await {
                println!("could not reply: {e}");
                continue;
            }
            send.finish()?;
        }

        println!("the sender left");
        Ok(())
    }
}
