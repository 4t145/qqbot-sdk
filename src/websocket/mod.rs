use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::model::*;
mod intends;
pub use intends::Intends;
mod payload;
pub use payload::*;
mod client;
pub use client::*;

impl From<UploadPayload> for Payload {
    fn from(upload: UploadPayload) -> Self {
        macro_rules! for_variant {
            ($val:expr; $($ident: ident),*) => {
                match $val {
                    $(
                        UploadPayload::$ident(data) => {
                            Payload {
                                opcode: Opcode::$ident,
                                data: serde_json::to_value(data).ok(),
                                tag: None,
                                seq: None
                            }
                        },
                    )*
                }
            };
        }
        for_variant!(upload; Heartbeat, Identify, Resume)
    }
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    Resume = 6,
    Reconnect = 7,
    // #[serde(rename = "Invalid Session")]
    InvalidSession = 9,
    Hello = 10,
    // #[serde(rename = "Heartbeat ACK")]
    HeartbeatAck = 11,
    // #[serde(rename = "HTTP Callback ACK")]
    HttpCallbackAck = 12,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "tag", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Event {
    MessageCreate(Box<MessageBotRecieved>),
    MessageDelete(Box<MessageDeleted>),
    PublicMessageDelete(Box<MessageDeleted>),
    AtMessageCreate(Box<MessageBotRecieved>),
    MessageAuditPass(Box<MessageAudited>),
    MessageAuditReject(Box<MessageAudited>),
    Ready(Box<Ready>),
    Resumed(String),
    MessageReactionAdd(Box<MessageReaction>),
    MessageReactionRemove(Box<MessageReaction>),
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ready {
    pub version: i32,
    pub session_id: String,
    pub user: User,
    pub shard: Option<[u32; 2]>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Identify {
    pub token: String,
    pub intents: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard: Option<[u32; 2]>,
    pub properties: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Resume {
    pub token: String,
    pub session_id: String,
    pub seq: u32,
}
