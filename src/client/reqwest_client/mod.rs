use std::sync::Arc;

use crate::api::{Authorization, Api};
use reqwest::{Url, ClientBuilder};

#[derive(Clone, Debug)]
pub struct ApiClient {
    client: reqwest::Client,
    auth_header: String
}

impl ApiClient {
    /// 仅仅提供授权，构建一个默认的客户端
    pub fn new(auth: Authorization) -> Self {
        let client = ClientBuilder::new().https_only(true).build().unwrap();
        let auth_header = auth.header();
        Self {
            client,
            auth_header,
        }
    }

    /// 自己提供一个客户端
    pub fn from_client(client: reqwest::Client, auth: Authorization) -> Self {
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
    pub fn set_auth(&mut self, auth: Authorization) {
        let auth_header = auth.header();
        self.auth_header = auth_header;
    }

    /// 发送一个请求
    /// 
    /// 例子
    /// ```
    /// let resp = client.send::<Getway>::(&()).await?
    /// ```
    pub async fn send<A:Api>(&self, request: &A::Request) -> Result<A::Response, reqwest::Error> {
        let url = Url::parse(format!("{}{}", env!("DOMAIN"), A::PATH).as_str()).unwrap();
        let resp = self.client
            .request(A::METHOD, url)
            .header(http::header::AUTHORIZATION, self.auth_header.as_str())
            .json(request)
            .send().await?;
        resp.json::<A::Response>().await
    }
}