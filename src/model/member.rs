use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::User;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Member {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 用户的频道基础信息，只有成员相关接口中会填充此信息
    pub user: Option<User>,
    /// 用户的昵称
    pub nick: String,
    #[serde(default)]
    /// 用户在频道内的身份组ID, 默认值可参考DefaultRoles
    pub roles: Vec<String>,
    /// 用户加入频道的时间
    pub joined_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MemberWithGuildID {
    #[serde(flatten)]
    /// 成员
    pub member: Member,
    /// 频道id
    pub guild_id: String,
}
