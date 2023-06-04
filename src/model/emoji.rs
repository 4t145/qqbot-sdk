use serde::{Deserialize, Serialize};
mod raw_emoji;
mod system_emoji;

pub use raw_emoji::*;
pub use system_emoji::*;

use crate::api::reaction::EmojiReactionDescriptor;

use super::MessageBotRecieved;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Deserialize, Serialize)]
#[repr(u32)]
#[serde(try_from = "EmojiJson", into = "EmojiJson")]
pub enum Emoji {
    System(u32) = 1,
    Raw(u32) = 2,
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

#[cfg(test)]
pub mod tests {
    use super::*;
    const TEST_CASE_PAIRS: &[(Emoji, &str)] = &[
        (Emoji::System(4), r#"{"type":1,"id":"4"}"#),
        (Emoji::Raw(127801), r#"{"type":2,"id":"127801"}"#),
    ];
    #[test]
    fn deserialize_test() {
        for (emoji, json) in TEST_CASE_PAIRS {
            let e: Emoji = serde_json::from_str(json).unwrap();
            assert_eq!(emoji, &e);
        }
    }

    #[test]
    fn serialize_test() {
        for (emoji, json) in TEST_CASE_PAIRS {
            let e = serde_json::to_string(emoji).unwrap();
            assert_eq!(json, &e);
        }
    }
}
