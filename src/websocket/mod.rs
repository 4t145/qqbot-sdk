use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::model::*;
mod intends;
pub use intends::Intends;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Payload {
    #[serde(rename = "op")]
    opcode: Opcode,
    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    seq: Option<u32>,
    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(into = "Payload")]
pub enum UploadPayload {
    Heartbeat(Option<u32>),
    Identify(Identify),
    Resume(Resume),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(from = "Payload")]
pub enum DownloadPayload {
    Dispatch { event: Box<Event>, seq: u32 },
    Heartbeat,
    Reconnect,
    InvalidSession,
    Hello { heartbeat_interval: u64 },
    HeartbeatAck,
}

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

impl From<Payload> for DownloadPayload {
    fn from(payload: Payload) -> Self {
        match payload.opcode {
            Opcode::Dispatch => {
                dbg!(&payload);
                let event = Box::new(
                    serde_json::from_value::<Event>(json!({
                        "tag": payload.tag,
                        "data": payload.data,
                    }))
                    .unwrap(),
                );
                DownloadPayload::Dispatch {
                    event,
                    seq: payload.seq.unwrap_or_default(),
                }
            }
            Opcode::Heartbeat => DownloadPayload::Heartbeat,
            Opcode::Reconnect => DownloadPayload::Reconnect,
            Opcode::InvalidSession => DownloadPayload::InvalidSession,
            Opcode::Hello => DownloadPayload::Hello {
                heartbeat_interval: payload
                    .data
                    .unwrap()
                    .get("heartbeat_interval")
                    .unwrap()
                    .as_u64()
                    .unwrap(),
            },
            Opcode::HeartbeatAck => DownloadPayload::HeartbeatAck,
            code => panic!("recieve websocket payload of unsupport opcode {code:?}"),
        }
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
    AtMessageCreate(Box<MessageRecieved>),
    MessageAuditPass(Box<MessageAudited>),
    MessageAuditReject(Box<MessageAudited>),
    Ready(Box<Ready>),
    Resumed(String),
    MessgaeReactionAdd(Box<MessageReaction>),
    MessgaeReactionRemove(Box<MessageReaction>),
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
