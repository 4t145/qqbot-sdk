use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::model::*;
mod payload;
pub use payload::*;

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum Opcode {
    Dispatch = 0,
    // #[serde(rename = "HTTP Callback ACK")]
    HttpCallbackAck = 12,
    HttpCallbackValidation = 13,
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Dispatch => write!(f, "`Dispatch`({})", *self as u8),
            Opcode::HttpCallbackAck => write!(f, "`HTTP Callback ACK`({})", *self as u8),
            Opcode::HttpCallbackValidation => {
                write!(f, "`HTTP Callback Validation`{}", *self as u8)
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "kind", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Event {
    MessageCreate(Arc<MessageBotRecieved>),
    MessageDelete(Arc<MessageDeleted>),
    PublicMessageDelete(Arc<MessageDeleted>),
    AtMessageCreate(Arc<MessageBotRecieved>),
    MessageAuditPass(Arc<MessageAudited>),
    MessageAuditReject(Arc<MessageAudited>),
    MessageReactionAdd(Arc<MessageReaction>),
    MessageReactionRemove(Arc<MessageReaction>),
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
