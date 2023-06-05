use super::*;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use chrono::{DateTime, Utc};
mod markdown;
pub use markdown::*;
pub use message_id::*;
mod message_content;
pub use message_content::*;
mod inline_key_board;
mod message_id;
#[cfg(test)]
mod tests;
pub use inline_key_board::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageDescriptor {
    /// 子频道id
    channel_id: u64,
    /// 消息id
    message_id: MessageId,
}

impl MessageDescriptor {
    pub fn into_sub_path(&self) -> String {
        format!("channels/{}/messages/{}", self.channel_id, self.message_id)
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageBotRecieved {
    /// 消息 id， 这里是64*3 = 192为十六进制数字
    pub id: MessageId,
    #[serde_as(as = "DisplayFromStr")]
    /// 子频道 id
    pub channel_id: ChannelId,
    #[serde_as(as = "DisplayFromStr")]
    /// 频道 id
    pub guild_id: GuildId,
    #[serde(default)]
    /// 消息内容
    pub content: String,
    #[serde(default)]
    /// 消息创建时间
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    /// 消息编辑时间
    pub edited_timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    /// 是否是@全员消息
    pub mention_everyone: bool,
    /// 消息创建者
    pub author: User,
    #[serde(default)]
    /// 附件
    pub attachments: Vec<MessageAttachment>,
    #[serde(default)]
    /// embed
    pub embeds: Vec<MessageEmbed>,
    #[serde(default)]
    /// 消息中@的人
    pub mentions: Vec<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 消息创建者的member信息
    pub member: Option<Member>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// ark消息
    pub ark: Option<MessageArk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 用于消息间的排序，seq 在同一子频道中按从先到后的顺序递增，不同的子频道之间消息无法排序。(目前只在消息事件中有值，2022年8月1日 后续废弃)
    pub seq: Option<u32>,
    #[serde_as(as = "DisplayFromStr")]
    /// 子频道消息 seq，用于消息间的排序，seq 在同一子频道中按从先到后的顺序递增，不同的子频道之间消息无法排序
    pub seq_in_channel: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// 引用消息对象
    pub message_reference: Option<MessageReference>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    /// 用于私信场景下识别真实的来源频道id
    pub src_guild_id: Option<u64>,
}

impl From<MessageBotRecieved> for MessageAudited {
    fn from(val: MessageBotRecieved) -> Self {
        MessageAudited {
            channel_id: val.channel_id,
            guild_id: val.guild_id,
            audit_id: Default::default(),
            message_id: Some(val.id),
            audit_time: Utc::now(),
            create_time: val.timestamp,
            seq_in_channel: val.seq_in_channel,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageDeleted {
    pub message: MessageDeleteMessage,
    pub op_user: User,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageDeleteMessage {
    pub id: MessageId,
    #[serde_as(as = "DisplayFromStr")]
    pub channel_id: ChannelId,
    #[serde_as(as = "DisplayFromStr")]
    pub guild_id: GuildId,
    pub author: User,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageGuildRecieved {
    guild_id: GuildId,
    #[serde(flatten)]
    inner: MessageChannelRecieved,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageChannelRecieved {
    channel_id: ChannelId,
    #[serde(flatten)]
    inner: MessageRecieved,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageRecieved {
    /// 消息 id， 这里是64*3 = 192为十六进制数字
    pub id: MessageId,
    /// 消息内容
    pub content: String,
    /// 消息创建时间
    pub timestamp: DateTime<Utc>,
    /// 消息编辑时间
    pub edited_timestamp: DateTime<Utc>,
    #[serde(default)]
    /// 是否是@全员消息
    pub mention_everyone: bool,
    /// 消息创建者
    pub author: User,
    #[serde(default)]
    /// 附件
    pub attachments: Vec<MessageAttachment>,
    #[serde(default)]
    /// embed
    pub embeds: Vec<MessageEmbed>,
    #[serde(default)]
    /// 消息中@的人
    pub mentions: Vec<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 消息创建者的member信息
    pub member: Option<Member>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// ark消息
    pub ark: Option<MessageArk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 用于消息间的排序，seq 在同一子频道中按从先到后的顺序递增，不同的子频道之间消息无法排序。(目前只在消息事件中有值，2022年8月1日 后续废弃)
    pub seq: Option<u32>,
    #[serde_as(as = "DisplayFromStr")]
    /// 子频道消息 seq，用于消息间的排序，seq 在同一子频道中按从先到后的顺序递增，不同的子频道之间消息无法排序
    pub seq_in_channel: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// 引用消息对象
    pub message_reference: Option<MessageReference>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    /// 用于私信场景下识别真实的来源频道id
    pub src_guild_id: Option<u64>,
}
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MessageSend<'a> {
    pub message_reference: Option<MessageReference>,
    pub content: Option<&'a str>,
    pub embed: Option<MessageEmbed>,
    pub ark: Option<MessageArk>,
    pub image: Option<&'a str>,
    pub markdown: Option<MessageMarkdown>,
    pub msg_id: Option<MessageId>,
    pub event_id: Option<&'a str>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageEmbed {
    /// 标题
    pub title: String,
    /// 消息弹窗内容
    pub prompt: String,
    /// 缩略图
    pub thumbnail: MessageEmbedThumbnail,
    /// 字段数据
    pub fields: Vec<MessageEmbedField>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageEmbedThumbnail {
    /// 图片地址
    url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageEmbedField {
    /// 字段名
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageAttachment {
    /// 下载地址
    url: String,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MessageReference {
    #[serde_as(as = "DisplayFromStr")]
    /// 需要引用回复的消息 id
    pub message_id: u64,
    /// 是否忽略获取引用消息详情错误，默认否
    ignore_get_message_error: bool,
}

impl MessageReference {
    pub fn new(message_id: u64) -> Self {
        Self {
            message_id,
            ignore_get_message_error: false,
        }
    }
    pub fn ignore_get_message_error(&mut self, ignore_get_message_error: bool) -> &mut Self {
        self.ignore_get_message_error = ignore_get_message_error;
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageArk {
    /// ark模板id（需要先申请）
    pub template_id: i32,
    /// kv值列表
    pub kv: Vec<MessageArkKv>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageArkKv {
    /// key
    key: String,
    /// value
    value: String,
    /// arkobj类型的数组 ark obj类型的列表
    obj: Vec<MessageArkObj>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageArkObj {
    /// ark objkv列表
    obj_kv: Vec<MessageArkObjKv>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageArkObjKv {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageDelete {
    /// 被删除的消息内容
    message: MessageBotRecieved,
    /// 执行删除操作的用户
    op_user: User,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageKeyboard {
    Template { id: String },
    Content { content: InlineKeyboard },
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct MessageAudited {
    // #[serde_as(as = "DisplayFromStr")]
    /// 消息审核 id
    pub audit_id: String,
    // #[serde_as(as = "DisplayFromStr")]
    /// 消息 id，只有审核通过事件才会有值
    pub message_id: Option<MessageId>,
    #[serde_as(as = "DisplayFromStr")]
    /// 频道 id
    pub guild_id: u64,
    #[serde_as(as = "DisplayFromStr")]
    /// 子频道 id
    pub channel_id: u64,
    /// 消息审核时间
    pub audit_time: DateTime<Utc>,
    /// 消息创建时间
    pub create_time: DateTime<Utc>,
    #[serde_as(as = "DisplayFromStr")]
    /// 子频道消息 seq，用于消息间的排序，seq 在同一子频道中按从先到后的顺序递增，不同的子频道之间消息无法排序
    pub seq_in_channel: u64,
}
