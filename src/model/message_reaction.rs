use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::{emoji::Emoji, ChannelId, GuildId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageReaction {
    pub user_id: String,
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub target: String,
    pub emoji: Emoji,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
#[non_exhaustive]
pub enum ReactionTargetType {
    /// 消息
    Message = 0,
    /// 帖子
    Post = 1,
    /// 评论
    Comment = 2,
    /// 回复
    Reply = 3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionTarget {
    pub id: String,
    pub r#type: ReactionTargetType,
}
