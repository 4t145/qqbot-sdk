use serde::Serialize;

use super::Api;

pub struct GetGuild;

#[derive(Debug, Serialize)]
pub struct GetGuildRequest {
    #[serde(skip)]
    pub guild_id: u64,
}

impl Api for GetGuild {
    type Request = GetGuildRequest;

    type Response = crate::model::Guild;

    const METHOD: http::Method = http::Method::GET;

    fn path(request: &Self::Request) -> String {
        format!("/guilds/{}", request.guild_id)
    }
}
