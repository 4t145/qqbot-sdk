pub mod app;
pub mod guild;
pub mod message;
pub mod reaction;
pub mod user;
pub mod websocket;
use std::fmt::Display;

use http::{
    Method, Request,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};

pub trait Api {
    type Request: Serialize;
    type Response: for<'a> Deserialize<'a>;
    const METHOD: Method;
    const PATH: &'static str = "";
    fn path(_request: &Self::Request) -> impl std::fmt::Display {
        Self::PATH
    }
}

pub fn json_request<A: Api>(request: &A::Request, base_url: &str, auth: &str) -> Request<Vec<u8>> {
    let body = serde_json::to_vec::<A::Request>(request).expect("fail to serialize json request");
    Request::builder()
        .uri(format!("{}{}", base_url, A::path(request)))
        .header(AUTHORIZATION, auth)
        .header(CONTENT_TYPE, "application/json")
        .method(A::METHOD)
        .body(body)
        .expect("fail to build json request")
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
        if let Some(data) = &self.data {
            f.write_str("\n")?;
            f.write_str(data.to_string().as_str())
        } else {
            f.write_str("\n没有数据")
        }
    }
}

impl Drop for ResponseFail {
    fn drop(&mut self) {
        tracing::error!("{}", self.message);
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
                message: format!("无法解析\n{}", v),
                code: u32::MAX,
                data: None,
            }),
        }
    }
}
