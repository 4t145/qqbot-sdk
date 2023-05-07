use serde::{ser::SerializeStruct, Deserialize, Serialize, de::Visitor};
mod raw_emoji;
mod system_emoji;

pub use raw_emoji::*;
pub use system_emoji::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u32)]
pub enum Emoji {
    System(u32) = 1,
    Raw(u32) = 2,
}

impl Emoji {
    pub fn into_sub_path(self) -> String {
        match self {
            Emoji::System(id) => format!("1/{}", id),
            Emoji::Raw(id) => format!("2/{}", id),
        }
    }
}

impl Serialize for Emoji {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Emoji", 2)?;
        let (tp, id) = match self {
            Emoji::System(emoji) => (1, emoji.to_string()),
            Emoji::Raw(emoji) => (2, emoji.to_string()),
        };
        s.serialize_field("type", &tp)?;
        s.serialize_field("id", &id)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Emoji {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
    D: serde::Deserializer<'de> {
        struct EmojiVisitor;
        
        impl<'de> Visitor<'de> for EmojiVisitor {
            type Value = Emoji;
        
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Emoji")
            }
        
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>, {
                let mut tp: Option<u32> = None;
                let mut id: Option<String> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "type" => {
                            if tp.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            tp = Some(map.next_value()?);
                        }
                        "id" => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let tp = tp.ok_or_else(|| serde::de::Error::missing_field("type"))?;
                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                match tp {
                    1 => Ok(Emoji::System(id.parse::<u32>().map_err(|e| serde::de::Error::custom(format!("cannot parse system emoji id <{id}>: {e}")))?)),
                    2 => Ok(Emoji::Raw(id.parse::<u32>().map_err(|e| serde::de::Error::custom(format!("cannot parse raw emoji id <{id}>: {e}")))?)),
                    _ => Err(serde::de::Error::custom(format!("unknown emoji type <{tp}>"))),
                }
            }
        }
        deserializer.deserialize_struct("Emoji", &["type", "id"], EmojiVisitor)
    }
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