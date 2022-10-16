use serde::{Serialize, Deserialize};
use http::{Method, header::{AUTHORIZATION, CONTENT_TYPE}, Request};
pub mod user;
pub mod websocket;
pub mod message;
/// 正式环境域名
pub const DOMAIN:&'static str = "https://api.sgroup.qq.com";
/// 沙箱环境域名
pub const SANDBOX_DOMAIN:&'static str = "https://sandbox.api.sgroup.qq.com";

#[derive(Clone, Debug)]
/// 授权
pub enum Authorization<'a> {
    Bot {
        app_id: &'a str,
        token: &'a str
    },
    Bearer {
        token: &'a str
    }
}

impl<'a> Into<String> for Authorization<'a> {
    fn into(self) -> String {
        self.token()
    }
}

impl<'a> Authorization<'a> {
    pub fn token(&self) -> String {
        match self {
            Authorization::Bot { app_id, token } => format!("{app_id}.{token}"),
            Authorization::Bearer { token } => token.to_string(),
        }
    }

    pub fn header(&self) -> String {
        match self {
            Authorization::Bot { app_id, token } => format!("Bot {app_id}.{token}"),
            Authorization::Bearer { token } => format!("Bearer {token}"),
        }
    }
}
pub trait Api {
    type Request: Serialize;
    type Response: for <'a> Deserialize<'a>;
    const METHOD: Method;
    const PATH: &'static str = "";
    fn path(_request: & Self::Request) -> String {
        return Self::PATH.to_string()
    }
}

pub fn json_request<A:Api>(request: &A::Request, auth: &str) -> Request<Vec<u8>> {
    let body = serde_json::to_vec::<A::Request>(request).unwrap();
    Request::builder()
        .uri(format!("{}{}", env!("DOMAIN"), A::path(request)))
        .header(AUTHORIZATION, auth)
        .header(CONTENT_TYPE, "application/json")
        .method(A::METHOD)
        .body(body).unwrap()
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseFail {
    pub message: String,
    pub code: u32,
    pub data: serde_json::Value
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Response<T> {
    Ok(T),
    Fail(ResponseFail),
    CannotParse(serde_json::Value)
}

impl<T> Response<T> {
    #[inline]
    pub fn as_result(self) -> Result<T, ResponseFail> {
        match self {
            Response::Ok(v) => Ok(v),
            Response::Fail(f) => Err(f),
            Response::CannotParse(v) => Err( ResponseFail {
                message: "无法解析".to_string(),
                code: u32::MAX,
                data: v
            }),
        }
    }
}