mod protocol;
mod receiver;
mod sender;

use iroh::{Endpoint, endpoint::presets};
use std::time::Duration;
use tokio::time::timeout;

const ALPN: &[u8] = b"iroh-file/0";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let peer = std::env::args().nth(1);
    let path = std::env::args().nth(2);

    let endpoint = Endpoint::bind(presets::N0).await?;
    wait_until_online(&endpoint).await;

    match peer {
        None => receiver::listen(endpoint).await?,
        Some(id) => {
            let path = path.expect("use like this: cargo run -- <id> <path>");
            sender::send_file(endpoint, id, path).await?
        }
    }

    Ok(())
}

async fn wait_until_online(endpoint: &Endpoint) {
    while timeout(Duration::from_secs(5), endpoint.online())
        .await
        .is_err()
    {
        println!("no connection, check wifi (retrying all 5 seconds)...");
        // im moment braucht es noch internet zugang.
        // Werden wir später ändern oder auch nicht je nach dem müssen wir besprechen.
    }
}
