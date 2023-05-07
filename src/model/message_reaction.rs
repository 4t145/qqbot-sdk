use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

use super::emoji::Emoji;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageReaction {
    pub user_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub target: String,
    pub emoji: Emoji
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
    Reply = 3
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionTarget {
    pub id: String,
    pub r#type: ReactionTargetType
}