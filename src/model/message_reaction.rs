use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::{serde_as, DisplayFromStr};

use super::{emoji::Emoji, ChannelId, GuildId, MessageId};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageReaction {
    pub user_id: String,
    #[serde_as(as = "DisplayFromStr")]
    pub guild_id: GuildId,
    #[serde_as(as = "DisplayFromStr")]
    pub channel_id: ChannelId,
    pub target: ReactionTarget,
    pub emoji: Emoji,
}

impl MessageReaction {
    pub fn into_unique_id(&self) -> String {
        format!("{g}/{c}/{u}/{t}/{e}", g=self.guild_id, c=self.channel_id, u=self.user_id, t=self.target.id, e=self.emoji)
    }
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReactionTarget {
    pub id: MessageId,
    pub r#type: ReactionTargetType,
}
