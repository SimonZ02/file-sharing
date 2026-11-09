use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    Offer { file_name: String, size: u64 },
    Answer { accepted: bool },
}