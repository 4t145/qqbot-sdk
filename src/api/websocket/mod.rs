use super::*;
use serde::{Deserialize};
use http::Method;
pub struct Gateway;
#[derive(Deserialize, Debug, Clone)]
pub struct GatewayResponse {
    pub url: String
}

impl Api for Gateway {
    type Request = ();

    type Response = GatewayResponse;

    const METHOD: Method = Method::GET;

    const PATH: &'static str = "/gateway";
}