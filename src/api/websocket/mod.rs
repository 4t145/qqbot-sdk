use super::*;
use http::Method;
use serde::Deserialize;
pub struct Gateway;
#[derive(Deserialize, Debug, Clone)]
pub struct GatewayResponse {
    pub url: String,
}

impl Api for Gateway {
    type Request = ();

    type Response = GatewayResponse;

    const METHOD: Method = Method::GET;

    const PATH: &'static str = "/gateway";
}


pub struct GatewayBot;

#[derive(Deserialize, Debug, Clone)]
pub struct GatewayBotResponse {
    pub url: String,
    pub shards: u32,
    pub session_start_limit: SessionStartLimit,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SessionStartLimit {
    pub total: u32,
    pub remaining: u32,
    /// in ms
    pub reset_after: u32,
    pub max_concurrency: u32,
}

impl Api for GatewayBot {
    type Request = ();

    type Response = GatewayBotResponse;

    const METHOD: Method = Method::GET;

    const PATH: &'static str = "/gateway/bot";
}
