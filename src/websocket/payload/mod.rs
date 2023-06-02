mod upload;
mod download;
use serde::{Serialize, Deserialize};
pub use upload::*;
pub use download::*;

use super::Opcode;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Payload {
    #[serde(rename = "op")]
    pub(crate) opcode: Opcode,
    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    pub(crate) seq: Option<u32>,
    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub(crate) tag: Option<String>,
    #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<serde_json::Value>,
}

