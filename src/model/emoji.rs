use std::str::FromStr;

use serde::{Deserialize, Serialize};
mod raw_emoji;
mod system_emoji;

pub use raw_emoji::*;
pub use system_emoji::*;

use crate::http::api::reaction::EmojiReactionDescriptor;

use super::MessageBotRecieved;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Deserialize, Serialize)]
#[repr(u32)]
#[serde(try_from = "EmojiJson", into = "EmojiJson")]
pub enum Emoji {
    System(u32) = 1,
    Raw(u32) = 2,
}

impl ToString for Emoji {
    fn to_string(&self) -> String {
        match self {
            Emoji::System(id) => format!("1:{}", id),
            Emoji::Raw(id) => format!("2:{}", id),
        }
    }
}

impl FromStr for Emoji {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(':');
        let tp = iter
            .next()
            .ok_or_else(|| format!("cannot parse emoji type from <{s}>"))?
            .parse::<u32>()
            .map_err(|e| format!("cannot parse emoji type from <{s}>: {e}"))?;
        let id = iter
            .next()
            .ok_or_else(|| format!("cannot parse emoji id from <{s}>"))?
            .parse::<u32>()
            .map_err(|e| format!("cannot parse emoji id from <{s}>: {e}"))?;
        match tp {
            1 => Ok(Emoji::System(id)),
            2 => Ok(Emoji::Raw(id)),
            _ => Err(format!("unknown emoji type <{tp}>")),
        }
    }
}

impl TryFrom<EmojiJson> for Emoji {
    type Error = String;

    fn try_from(value: EmojiJson) -> Result<Self, Self::Error> {
        match value.r#type {
            1 => Ok(Emoji::System(value.id.parse().map_err(|e| {
                format!("cannot parse system emoji id <{id}>: {e}", id = value.id)
            })?)),
            2 => Ok(Emoji::Raw(value.id.parse().map_err(|e| {
                format!("cannot parse raw emoji id <{id}>: {e}", id = value.id)
            })?)),
            _ => Err("Unknown emoji type".to_owned()),
        }
    }
}

impl From<Emoji> for EmojiJson {
    fn from(val: Emoji) -> Self {
        match val {
            Emoji::System(id) => EmojiJson {
                r#type: 1,
                id: id.to_string(),
            },
            Emoji::Raw(id) => EmojiJson {
                r#type: 2,
                id: id.to_string(),
            },
        }
    }
}

impl Emoji {
    pub fn into_sub_path(self) -> String {
        match self {
            Emoji::System(id) => format!("1/{}", id),
            Emoji::Raw(id) => format!("2/{}", id),
        }
    }

    pub fn react_to(self, message: &MessageBotRecieved) -> EmojiReactionDescriptor {
        EmojiReactionDescriptor {
            channel_id: message.channel_id,
            message_id: message.id,
            emoji: self,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct EmojiJson {
    r#type: u32,
    id: String,
}


