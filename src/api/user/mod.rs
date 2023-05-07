use serde::Serialize;

use crate::model::User;

use super::Api;

pub struct GetMe;

impl Api for GetMe {
    type Request = ();

    type Response = User;

    const METHOD: http::Method = http::Method::GET;

    const PATH: &'static str = "/users/@me";
}

pub struct GetMyGuilds;

#[derive(Debug, Serialize, Default)]
pub struct GetMyGuildsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 每次拉取多少条数据, 默认 100, 最大 100
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 读此 guild id 之前的数据, before 设置时， 先反序，再分页
    pub before: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 读此 guild id 之后的数据, after 和 before 同时设置时， after 参数无效
    pub after: Option<u64>,
}

impl GetMyGuildsRequest {
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn before(mut self, before: u64) -> Self {
        self.before = Some(before);
        self
    }
    pub fn after(mut self, after: u64) -> Self {
        self.after = Some(after);
        self
    }
}

impl Api for GetMyGuilds {
    type Request = GetMyGuildsRequest;

    type Response = Vec<crate::model::Guild>;

    const METHOD: http::Method = http::Method::GET;

    const PATH: &'static str = "/users/@me/guilds";
}
