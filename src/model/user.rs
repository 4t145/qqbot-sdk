use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct User {
    #[serde_as(as = "DisplayFromStr")]
    pub id: u64,
    pub username: String,
    pub bot: bool,
    pub avatar: Option<String>,
    #[serde(skip_serializing_if ="Option::is_none")]
    pub union_openid: Option<String>,
    #[serde(skip_serializing_if ="Option::is_none")]
    pub union_user_account: Option<String>
}