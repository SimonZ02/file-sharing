use iroh::endpoint::{RecvStream, SendStream};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Offer { file_name: String, size: u64 },
    Answer { accepted: bool },
}

pub async fn send_message(send: &mut SendStream, message: &Message) -> anyhow::Result<()> {
    let bytes = postcard::to_allocvec(message)?;
    let bytes_length = (bytes.len() as u32).to_be_bytes();
    send.write_all(&bytes_length).await?;
    send.write_all(&bytes).await?;

    Ok(())
}

pub async fn read_message(recv: &mut RecvStream) -> anyhow::Result<Message> {

    let mut bytes_length = [0u8; 4];
    recv.read_exact(&mut bytes_length).await?;

    let length = u32::from_be_bytes(bytes_length) as usize;

    if length > 1000 {
        return Err(anyhow::anyhow!("message too big: {length} bytes"));
    }

    let mut bytes = vec![0u8; length];
    recv.read_exact(&mut bytes).await?;

    let message = postcard::from_bytes(&bytes)?;

    Ok(message)
}