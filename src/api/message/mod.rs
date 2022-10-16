use crate::model::{Message, MessageEmbed, MessageArk, MessageReference, MessageMarkdown};

use super::{Api, Response};
use serde::{Serialize, Deserialize};

pub struct GetMessage;

#[derive(Serialize)]
pub struct GetMessageRequest {
    /// 子频道id
    channel_id: u64,
    /// 消息id
    message_id: String,
}

impl Api for GetMessage {
    type Request = GetMessageRequest;

    type Response = Response<Message>;

    const METHOD: http::Method = http::Method::GET;

    fn path<'a>(request: &'a Self::Request) -> String {
        format!("/channels/{}/messages/{}", request.channel_id, request.message_id)
    }
}

pub struct PostMessage;

#[derive(Serialize, Default)]
pub struct PostMessageRequest {
    /// 子频道id
    pub channel_id: u64,
    /// 消息id
    pub message_id: String,
    ///选填，消息内容，文本内容，支持内嵌格式
    pub content: Option<String>,
    /// 选填，embed 消息，一种特殊的 ark，详情参考Embed消息
    pub embed: Option<MessageEmbed>,
    /// ark消息对象 	选填，ark 消息
    pub ark: Option<MessageArk>,
    /// 引用消息对象 	选填，引用消息
    pub message_reference: Option<MessageReference>,
    /// 选填，图片url地址，平台会转存该图片，用于下发图片消息
    pub image: Option<String>,
    /// 选填，要回复的消息id(Message.id), 在 AT_CREATE_MESSAGE 事件中获取。
    pub msg_id: Option<String>,
    /// 选填，要回复的事件id, 在各事件对象中获取。
    pub event_id: Option<String>,
    /// markdown 消息对象 	选填，markdown 消息
    pub markdown: Option<MessageMarkdown>,
}

impl Api for PostMessage {
    type Request = PostMessageRequest;

    type Response = Response<Message>;

    const METHOD: http::Method = http::Method::GET;

    fn path<'a>(request: &'a Self::Request) -> String {
        format!("/channels/{}/messages/{}", request.channel_id, request.message_id)
    }
}