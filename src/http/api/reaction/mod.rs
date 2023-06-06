use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::model::{Emoji, MessageId, User};

use super::Api;

/// 发表表情表态
pub struct SendEmojiReaction;

/// 删除表情表态
pub struct DeleteEmojiReaction;

/// 获取表情表态用户列表
pub struct GetEmojiReactionUserList<'a> {
    marker: PhantomData<&'a ()>,
}

///  发表表情表态
#[derive(Debug, Serialize)]
pub struct EmojiReactionDescriptor {
    /// 子频道id
    pub channel_id: u64,
    /// 消息id
    pub message_id: MessageId,
    /// emoji
    pub emoji: Emoji,
}

impl Api for SendEmojiReaction {
    type Request = EmojiReactionDescriptor;

    type Response = ();

    const METHOD: http::Method = http::Method::PUT;

    fn path(request: &Self::Request) -> String {
        format!(
            "/channels/{}/messages/{}/reactions/{}",
            request.channel_id,
            request.message_id,
            request.emoji.into_sub_path()
        )
    }
}

/// 获取表情表态用户列表 请求
#[derive(Debug, Serialize)]
pub struct GetEmojiReactionUserListRequest<'a> {
    #[serde(skip)]
    /// Emoji描述符
    pub descriptor: &'a EmojiReactionDescriptor,
    /// 上次请求返回的cookie，第一次请求无需填写
    pub cookie: Option<String>,
    /// 每次拉取数量，默认20，最多50，只在第一次请求时设置
    pub limit: Option<u32>,
}

impl<'a> GetEmojiReactionUserListRequest<'a> {
    pub fn new(descriptor: &'a EmojiReactionDescriptor) -> Self {
        Self {
            descriptor,
            cookie: None,
            limit: Some(20),
        }
    }
    pub fn next(&mut self, cookie: String) -> &mut Self {
        self.limit = None;
        self.cookie = Some(cookie);
        self
    }
}
#[derive(Debug, Deserialize)]
/// 获取表情表态用户列表 响应
pub struct GetEmojiReactionUserListResponse {
    /// 用户对象列表
    pub users: Vec<User>,
    /// 下次请求的cookie，如果为空，则表示已经拉取完毕
    pub cookie: Option<String>,
    /// 是否已拉取完成到最后一页，true代表完成
    pub is_end: bool,
}

impl Api for DeleteEmojiReaction {
    type Request = EmojiReactionDescriptor;

    type Response = ();

    const METHOD: http::Method = http::Method::DELETE;

    fn path(request: &Self::Request) -> String {
        format!(
            "/channels/{}/messages/{}/reactions/{}",
            request.channel_id,
            request.message_id,
            request.emoji.into_sub_path()
        )
    }
}

impl<'a> Api for GetEmojiReactionUserList<'a> {
    type Request = GetEmojiReactionUserListRequest<'a>;

    type Response = GetEmojiReactionUserListResponse;

    const METHOD: http::Method = http::Method::GET;

    fn path(request: &Self::Request) -> String {
        format!(
            "/channels/{}/messages/{}/reactions/{}",
            request.descriptor.channel_id,
            request.descriptor.message_id,
            request.descriptor.emoji.into_sub_path()
        )
    }
}
