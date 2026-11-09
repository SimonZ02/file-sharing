use iroh::{Endpoint, EndpointId};

use crate::ALPN;

/*
Sender: has the file.
- searches for peers who are open to receive files.
*/

pub async fn send_file(endpoint: Endpoint, id: String, path: String) -> anyhow::Result<()> {
    let peer_id: EndpointId = id.parse()?;
    println!("connecting ...");

    let connection = endpoint.connect(peer_id, ALPN).await?;
    println!("connection successful");

    let bytes = tokio::fs::read(&path).await?;
    println!("sending {} bytes", bytes.len());

    let (mut send, mut recv) = connection.open_bi().await?;
    send.write_all(&bytes).await?;
    send.finish()?;

    let reply = recv.read_to_end(1000).await?;
    println!("reply: {}", String::from_utf8_lossy(&reply));

    endpoint.close().await;

    Ok(())
}