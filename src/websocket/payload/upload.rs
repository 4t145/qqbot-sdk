use serde::Serialize;

use crate::websocket::{Identify, Resume};

use super::Payload;



#[derive(Serialize, Clone, Debug)]
#[serde(into = "Payload")]
pub enum UploadPayload {
    Heartbeat(Option<u32>),
    Identify(Identify),
    Resume(Resume),
}