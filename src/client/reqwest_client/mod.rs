use std::sync::Arc;

use crate::api::{Authority, Api, Response};
use reqwest::{Url, ClientBuilder};

mod message;

#[derive(Clone, Debug)]
pub struct ApiClient {
    client: reqwest::Client,
    auth_header: String
}

impl ApiClient {
    /// 仅仅提供授权，构建一个默认的客户端
    pub fn new(auth: Authority) -> Self {
        let client = ClientBuilder::new().https_only(true).build().unwrap();
        let auth_header = auth.header();
        Self {
            client,
            auth_header,
        }
    }

    /// 自己提供一个客户端
    pub fn from_client(client: reqwest::Client, auth: Authority) -> Self {
        let auth_header = auth.header();
        Self {
            client,
            auth_header,
        }
    }

    /// 使用`Arc`包裹当前客户端
    #[inline]
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// 设置新授权
    pub fn set_auth(&mut self, auth: Authority) {
        let auth_header = auth.header();
        self.auth_header = auth_header;
    }

    /// 发送一个请求
    /// 
    /// 例子
    /// ```
    /// let resp = client.send::<Getway>::(&()).await?
    /// ```
    pub async fn send<A:Api>(&self, request: &A::Request) -> Result<Response<A::Response>, reqwest::Error> {
        let url = Url::parse(format!("{}{}", env!("DOMAIN"), A::path(request)).as_str()).unwrap();
        log::debug!("send request: {json} to {url}", json=serde_json::to_string_pretty(request).unwrap());
        let resp = self.client
            .request(A::METHOD, url)
            .header(http::header::AUTHORIZATION, self.auth_header.as_str())
            .json(request)
            .send().await?;
        resp.json::<Response<A::Response>>().await
    }
}