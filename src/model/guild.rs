use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

pub type GuildId = u64;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Guild {
    #[serde_as(as = "DisplayFromStr")]
    ///频道ID
    pub id: GuildId,
    ///频道名称
    pub name: String,
    ///频道头像地址
    pub icon: String,
    ///创建人用户ID
    pub owner_id: String,
    ///当前人是否是创建人
    pub owner: bool,
    ///成员数
    pub member_count: i32,
    ///最大成员数
    pub max_members: i32,
    ///描述
    pub description: String,
    ///加入时间
    pub joined_at: DateTime<Utc>,
}
