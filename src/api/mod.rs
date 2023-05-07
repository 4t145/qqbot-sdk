use std::fmt::Display;

use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Method, Request,
};
use serde::{Deserialize, Serialize};
pub mod guild;
pub mod message;
pub mod reaction;
pub mod user;
pub mod websocket;

#[derive(Clone, Debug)]
/// 授权
pub enum Authority<'a> {
    Bot { app_id: &'a str, token: &'a str },
    Bearer { token: &'a str },
}

impl<'a> From<Authority<'a>> for String {
    fn from(val: Authority<'a>) -> Self {
        val.token()
    }
}

impl<'a> Authority<'a> {
    pub fn new_bot(app_id: impl Into<&'a str>, token: impl Into<&'a str>) -> Self {
        Authority::Bot {
            app_id: app_id.into(),
            token: token.into(),
        }
    }
    pub fn new_bearer(token: impl Into<&'a str>) -> Self {
        Authority::Bearer {
            token: token.into(),
        }
    }
    pub fn token(&self) -> String {
        match self {
            Authority::Bot { app_id, token } => format!("Bot {app_id}.{token}"),
            Authority::Bearer { token } => format!("Bearer {token}"),
        }
    }

    pub fn header(&self) -> String {
        match self {
            Authority::Bot { app_id, token } => format!("Bot {app_id}.{token}"),
            Authority::Bearer { token } => format!("Bearer {token}"),
        }
    }
}
pub trait Api {
    type Request: Serialize;
    type Response: for<'a> Deserialize<'a>;
    const METHOD: Method;
    const PATH: &'static str = "";
    fn path(_request: &Self::Request) -> String {
        Self::PATH.to_string()
    }
}

pub fn json_request<A: Api>(request: &A::Request, auth: &str) -> Request<Vec<u8>> {
    let body = serde_json::to_vec::<A::Request>(request).unwrap();
    Request::builder()
        .uri(format!("{}{}", env!("DOMAIN"), A::path(request)))
        .header(AUTHORIZATION, auth)
        .header(CONTENT_TYPE, "application/json")
        .method(A::METHOD)
        .body(body)
        .unwrap()
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseFail {
    pub message: String,
    pub code: u32,
    pub data: Option<serde_json::Value>,
}

impl Display for ResponseFail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("请求失败：[{:4}]{}", self.code, self.message.as_str()).as_str())?;
        f.write_str("data: \n")?;
        f.write_str(serde_json::to_string_pretty(&self.data).unwrap().as_str())
    }
}

impl Drop for ResponseFail {
    fn drop(&mut self) {
        log::error!("{}", self.message);
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Response<T> {
    Ok(T),
    Fail(ResponseFail),
    CannotParse(serde_json::Value),
}

impl<T> Response<T> {
    #[inline]
    pub fn as_result(self) -> Result<T, ResponseFail> {
        match self {
            Response::Ok(v) => Ok(v),
            Response::Fail(f) => Err(f),
            Response::CannotParse(v) => Err(ResponseFail {
                message: format!("无法解析\n{}", serde_json::to_string_pretty(&v).unwrap()),
                code: u32::MAX,
                data: None,
            }),
        }
    }
}
