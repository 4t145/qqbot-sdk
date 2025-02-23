use std::{fmt::Display, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

/// message id, 三个u64组成，大端序
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MessageId(Arc<str>);

impl Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{:016x}{:016x}{:016x}", self.0[2], self.0[1], self.0[0])
        write!(f, "{}", self.0)
    }
}

impl FromStr for MessageId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("invalid message id (empty)");
        }
        Ok(MessageId(s.to_string().into()))
    }
}
// 64*3/4 = 48
impl Serialize for MessageId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MessageId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        MessageId::from_str(&s)
            .map_err(|e| serde::de::Error::custom(format!("invalid message id, {e}")))
    }
}
