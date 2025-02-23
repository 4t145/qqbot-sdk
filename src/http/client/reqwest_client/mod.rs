use std::{sync::Arc, time::Instant};

use crate::http::api::{Api, Response, app};
use http::HeaderValue;
use reqwest::{ClientBuilder, Url};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: Arc<str>,
    client_secret: Arc<str>,
    app_id: Arc<str>,
    auth_header: Arc<RwLock<Option<(HeaderValue, Instant)>>>,
}

impl ApiClient {
    /// 仅仅提供授权，构建一个默认的客户端
    pub fn new(secret: &str, app_id: &str, base_url: &str) -> Self {
        let client = ClientBuilder::new()
            .https_only(true)
            .build()
            .unwrap_or_default();
        Self {
            client,
            auth_header: Arc::new(RwLock::new(None)),
            client_secret: secret.into(),
            app_id: app_id.into(),
            base_url: base_url.into(),
        }
    }

    /// 自己提供一个客户端
    pub fn from_client(
        client: reqwest::Client,
        secret: String,
        app_id: &str,
        base_url: String,
    ) -> Self {
        Self {
            client,
            auth_header: Arc::new(RwLock::new(None)),
            client_secret: secret.into(),
            app_id: app_id.into(),
            base_url: base_url.into(),
        }
    }

    /// 刷新授权头
    pub async fn refresh_auth_header(&self) -> crate::Result<HeaderValue> {
        let auth_request = app::GetAccessTokenRequest {
            app_id: &self.app_id,
            client_secret: &self.client_secret,
        };
        let url = Url::parse(
            format!(
                "{}{}",
                self.base_url,
                app::GetAccessToken::path(&auth_request)
            )
            .as_str(),
        )
        .expect("invalid url, report this bug");
        // refresh token
        let resp = self
            .client
            .request(app::GetAccessToken::METHOD, url)
            .json(&auth_request)
            .send()
            .await
            .map_err(crate::Error::context("send request to get access token"))?;
        let resp = resp
            .json::<Response<app::GetAccessTokenResponse>>()
            .await
            .map_err(crate::Error::context("parse get access token response"))?;
        let response = resp
            .as_result()
            .map_err(|_| crate::Error::unexpected("get access token response is not success"))?;
        let token = response.access_token;
        let expire_at = Instant::now() + std::time::Duration::from_secs(response.expires_in.into());
        let new_header = format!("QQBot {}", token);
        let new_header = HeaderValue::from_str(new_header.as_str())
            .map_err(|_| crate::Error::unexpected("invalid new auth header"))?;
        let mut auth_header = self.auth_header.write().await;
        *auth_header = Some((new_header.clone(), expire_at));
        Ok(new_header)
    }
    /// 发送一个请求
    ///
    /// 例子
    /// ```rust,no_run,ignore
    /// let resp = client.send::<Getway>::(&()).await?
    /// ```
    pub async fn send<A: Api>(&self, request: &A::Request) -> crate::Result<Response<A::Response>> {
        // check if the token is expired

        let auth_header = {
            let auth_header = self.auth_header.read().await;
            if let Some((header_value, expire_at)) = auth_header.as_ref() {
                if *expire_at < Instant::now() {
                    // blocking get new token
                    self.refresh_auth_header().await?
                } else if *expire_at < Instant::now() + std::time::Duration::from_secs(60) {
                    let client = self.clone();
                    tokio::spawn(async move {
                        let _ = client.refresh_auth_header().await;
                    });
                    header_value.clone()
                } else {
                    header_value.clone()
                }
            } else {
                // blocking get new token
                self.refresh_auth_header().await?
            }
        };
        let url = Url::parse(format!("{}{}", self.base_url, A::path(request)).as_str())
            .expect("invalid url, report this bug");
        let resp = self
            .client
            .request(A::METHOD, url)
            .header(http::header::AUTHORIZATION, auth_header)
            .json(request)
            .send()
            .await
            .map_err(crate::Error::context("send request"))?;
        resp.json::<Response<A::Response>>()
            .await
            .map_err(crate::Error::context("parse response"))
    }
}
