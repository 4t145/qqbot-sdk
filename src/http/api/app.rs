use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::Api;

pub struct GetAccessToken<'a> {
    marker: std::marker::PhantomData<&'a ()>,
}
#[derive(Debug, Serialize)]
pub struct GetAccessTokenRequest<'a> {
    pub(crate) app_id: &'a str,
    pub(crate) client_secret: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde_as]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenResponse {
    pub(crate) access_token: String,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) expires_in: u32,
}

impl<'a> Api for GetAccessToken<'a> {
    type Request = GetAccessTokenRequest<'a>;

    type Response = GetAccessTokenResponse;

    const METHOD: http::Method = http::Method::POST;

    fn path(_request: &Self::Request) -> impl std::fmt::Display {
        "/app/access_token"
    }
}
