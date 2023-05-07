use crate::model::{
    MessageArk, MessageDescriptor, MessageEmbed, MessageId, MessageMarkdown, MessageRecieved,
    MessageReference, MessageSend,
};

use super::Api;
use serde::Serialize;

pub struct GetMessage;

impl Api for GetMessage {
    type Request = MessageDescriptor;

    type Response = MessageRecieved;

    const METHOD: http::Method = http::Method::GET;

    fn path(request: &Self::Request) -> String {
        format!("/{}", request.into_sub_path())
    }
}

pub struct PostMessage<'a> {
    marker: std::marker::PhantomData<&'a ()>,
}

#[derive(Serialize, Default, Debug)]
pub struct PostMessageRequest<'a> {
    #[serde(skip)]
    pub channel_id: u64,
    ///选填，消息内容，文本内容，支持内嵌格式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<&'a str>,
    /// 选填，embed 消息，一种特殊的 ark，详情参考Embed消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<&'a MessageEmbed>,
    /// ark消息对象     选填，ark 消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ark: Option<&'a MessageArk>,
    /// 引用消息对象     选填，引用消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<&'a MessageReference>,
    /// 选填，图片url地址，平台会转存该图片，用于下发图片消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<&'a str>,
    /// 选填，要回复的消息id(Message.id), 在 AT_CREATE_MESSAGE 事件中获取。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MessageId>,
    /// 选填，要回复的事件id, 在各事件对象中获取。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<&'a str>,
    /// markdown 消息对象     选填，markdown 消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<&'a MessageMarkdown>,
}

impl<'a> PostMessageRequest<'a> {
    pub fn new(channel_id: u64, message: &'a MessageSend) -> Self {
        Self {
            channel_id,
            content: message.content,
            embed: message.embed.as_ref(),
            ark: message.ark.as_ref(),
            message_reference: message.message_reference.as_ref(),
            image: message.image,
            msg_id: message.msg_id,
            event_id: message.event_id,
            markdown: message.markdown.as_ref(),
        }
    }
}

impl<'a> Api for PostMessage<'a> {
    type Request = PostMessageRequest<'a>;

    type Response = MessageRecieved;

    const METHOD: http::Method = http::Method::POST;

    fn path(request: &Self::Request) -> String {
        format!(
            "/channels/{channel_id}/messages",
            channel_id = request.channel_id
        )
    }
}

/// 撤回消息
pub struct DeleteMessage;

#[derive(Serialize)]
pub struct DeleteMessageRequest {
    #[serde(skip)]
    pub discriptor: MessageDescriptor,
    /// 选填，是否隐藏提示小灰条，true 为隐藏，false 为显示。默认为false
    pub hidetip: Option<bool>,
}

impl Api for DeleteMessage {
    type Request = MessageDescriptor;

    type Response = ();

    const METHOD: http::Method = http::Method::DELETE;

    fn path(request: &Self::Request) -> String {
        format!("/{}", request.into_sub_path())
    }
}
