use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use futures_util::Stream;
use tokio_util::sync::CancellationToken;

use crate::event::{EventStreamProvider, model::Event};
mod middleware;
mod service;
mod utils;

#[derive(Debug)]
pub struct WebHookService {
    pub(crate) rx: tokio::sync::mpsc::Receiver<Event>,
    pub(crate) bind: SocketAddr,
}

impl Stream for WebHookService {
    type Item = Event;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

impl EventStreamProvider for WebHookService {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "axum-webhook".into()
    }
}
#[derive(Debug, Clone)]
pub struct WebHookServiceApp {
    bot_secret: Arc<str>,
    event_tx: tokio::sync::mpsc::Sender<Event>,
    config: Arc<WebHookServiceConfig>,
}

#[derive(Debug, Clone)]
pub struct WebHookServiceConfig {
    body_size_limit: u32,
}

impl WebHookServiceApp {
    pub fn build_service(&self) -> crate::Result<Router<()>> {
        let app = self.clone();
        let router = axum::Router::new()
            .route("/", axum::routing::any(service::event_listen_service))
            .layer(axum::middleware::from_fn_with_state(
                app.clone(),
                middleware::signature_check,
            ))
            .with_state(app);
        Ok(router)
    }
}

pub struct WebHookServiceAppConfig {
    pub bind: SocketAddr,
    pub channel_size: usize,
    pub max_body_size: usize,
}

impl WebHookService {
    pub fn get_bind(&self) -> SocketAddr {
        self.bind
    }
    pub async fn run(
        config: WebHookServiceAppConfig,
        secret: &str,
        ct: CancellationToken,
    ) -> crate::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel(config.channel_size);
        let app = WebHookServiceApp {
            bot_secret: Arc::from(secret),
            event_tx: tx,
            config: Arc::new(WebHookServiceConfig {
                body_size_limit: 1024 * 1024,
            }),
        };
        let app = Arc::new(app);
        let tokio_tcp_listen = tokio::net::TcpListener::bind(config.bind)
            .await
            .map_err(crate::Error::context("failed to bind to address"))?;
        let service = app.build_service()?;
        tokio::spawn(async move {
            let result = axum::serve(tokio_tcp_listen, service)
                .with_graceful_shutdown(async move {
                    ct.cancelled().await;
                    tracing::info!("webhook service shutdown");
                })
                .await;
            if let Err(err) = result {
                tracing::error!("webhook service error: {:?}", err);
            }
        });
        Ok(Self {
            rx,
            bind: config.bind,
        })
    }
}
