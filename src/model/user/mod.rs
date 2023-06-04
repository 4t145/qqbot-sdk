#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq)]
pub struct User {
    #[serde_as(as = "DisplayFromStr")]
    pub id: u64,
    #[serde(default)]
    pub username: String,
    /// 妹说就是不是
    #[serde(default)]
    pub bot: bool,
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub union_openid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub union_user_account: Option<String>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
