use iroh::{Endpoint, EndpointId};

use crate::{
    ALPN,
    file_utils::check_file,
    protocol::{Message, read_message, send_message},
};

/*
Sender: has the file.
- searches for peers who are open to receive files.
*/

pub async fn send_file(endpoint: Endpoint, id: String, path: String) -> anyhow::Result<()> {
    let file_info = check_file(&path)?;
    let peer_id: EndpointId = id.parse()?;
    println!("connecting ...");

    let connection = endpoint.connect(peer_id, ALPN).await?;
    println!("connection successful");

    let file_size = file_info.size;

    println!("sending {} bytes", file_size);

    let file_name = file_info.name;

    let offer = Message::Offer {
        file_name: file_name,
        size: file_size,
    };

    println!("{:?}", offer);

    let (mut send, mut recv) = connection.open_bi().await?;

    send_message(&mut send, &offer).await?;

    let answer = match read_message(&mut recv).await {
        Ok(answer) => answer,
        Err(e) => {
            anyhow::bail!("could not read message: {e}");
        }
    };

    let accepted = match answer {
        Message::Answer { accepted } => accepted,
        _ => anyhow::bail!("expected an answer"),
    };

    if !accepted {
        println!("denied by receiver");
        endpoint.close().await;
        return Ok(());
    };

    let mut file = tokio::fs::File::open(&path).await?;

    tokio::io::copy(&mut file, &mut send).await?;
    send.finish()?;

    let reply = recv.read_to_end(1000).await?;
    println!("reply: {}", String::from_utf8_lossy(&reply));

    endpoint.close().await;

    Ok(())
}
