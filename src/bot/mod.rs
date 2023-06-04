mod dispacher;
mod handler;
mod message;
mod methods;
mod shard;
mod user;

use crate::{
    api::{
        websocket::{Gateway, GatewayBot},
        Authority,
    },
    client::{
        reqwest_client::ApiClient, tungstenite_client::TungsteniteConnection, ConnectConfig,
        ConnectType, Connection, ClientEvent,
    },
    model::Guild,
    websocket::{Event, Identify},
};
pub use message::*;
pub use shard::Shards;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle};
pub use user::*;

pub use self::handler::Handler;

// use self::dispacher::{EventDispatcher};

/// # Bot
/// ## Clone
/// 可以随意克隆bot，内部全部有arc包裹, 且不提供可变性
#[derive(Debug, Clone)]
pub struct Bot {
    api_client: Arc<ApiClient>,
    event_tx: Arc<tokio::sync::broadcast::Sender<ClientEvent>>,
    cache: BotCache,
    handlers: Arc<RwLock<HashMap<String, JoinHandle<()>>>>, // dispacher: Arc<RwLock<EventDispatcher>>,
}

#[derive(Debug, Clone, Default)]
pub struct BotCache {
    guilds: Arc<RwLock<HashMap<u64, Guild>>>,
    // users: Arc<RwLock<HashMap<u64, User>>>,
}
impl BotCache {
    pub async fn cache_guild(&self, guild: Guild) {
        self.guilds.write().await.insert(guild.id, guild);
    }
    pub async fn cache_many_guilds(&self, guilds: impl IntoIterator<Item = Guild>) {
        let mut guilds_cache = self.guilds.write().await;
        for guild in guilds {
            guilds_cache.insert(guild.id, guild);
        }
    }
    pub async fn get_guild(&self, id: u64) -> Option<Guild> {
        self.guilds.read().await.get(&id).cloned()
    }
    pub async fn get_guilds_count(&self) -> usize {
        self.guilds.read().await.len()
    }
}
#[derive(Debug, Clone, Default)]
pub struct BotBuilder<'a> {
    authority: Option<Authority<'a>>,
    shards: Option<Shards>,
    auto_shard: bool,
    intents: u32,
}

#[derive(Debug)]
pub enum BotBuildError {
    NoAuthority,
    ApiError(reqwest::Error),
    WsConnectError(crate::client::tungstenite_client::TungsteniteConnectionError),
}

impl<'a> BotBuilder<'a> {
    pub fn auth(self, authority: Authority<'a>) -> Self {
        Self {
            authority: Some(authority),
            ..self
        }
    }
    pub fn shards(self, shards: Shards) -> Self {
        Self {
            shards: Some(shards),
            ..self
        }
    }
    pub fn intents(self, intents: u32) -> Self {
        Self {
            intents: self.intents | intents,
            ..self
        }
    }
    pub fn auto_shard(self, auto_shard: bool) -> Self {
        Self { auto_shard, ..self }
    }
    pub async fn build(mut self) -> Result<Bot, BotBuildError> {
        let auth = self.authority.ok_or(BotBuildError::NoAuthority)?;
        let token = auth.token();
        let api_client = ApiClient::new(auth);
        let url = if self.auto_shard {
            let response = api_client
                .send::<GatewayBot>(&())
                .await
                .map_err(BotBuildError::ApiError)?
                .as_result()
                .unwrap();
            self.shards = Some(Shards::new_all(response.shards));
            response.url
        } else {
            api_client
                .send::<Gateway>(&())
                .await
                .map_err(BotBuildError::ApiError)?
                .as_result()
                .unwrap()
                .url
        };
        log::info!("ws gate url: {}", url);
        let (event_tx, _event_rx) = tokio::sync::broadcast::channel(128);
        if let Some(shards) = self.shards {
            let total = shards.total;
            for shard_idx in shards.using_shards {
                // 启动websocket client
                let identify = Identify {
                    token: token.clone(),
                    intents: self.intents,
                    shard: Some([shard_idx, total]),
                    properties: std::collections::HashMap::new(),
                };
                // ws连接设置
                let connect_option = ConnectConfig {
                    wss_gateway: url.clone(),
                    retry_times: 5,
                    retry_interval: tokio::time::Duration::from_secs(30),
                    identify,
                };
                let mut conn = TungsteniteConnection::new(connect_option, event_tx.clone());
                conn.connect()
                    .await
                    .map_err(BotBuildError::WsConnectError)?;
            }
        } else {
            // standalone
            let identify = Identify {
                token,
                intents: self.intents,
                shard: Some([0, 1]),
                properties: std::collections::HashMap::new(),
            };
            // ws连接设置
            let connect_option = ConnectConfig {
                wss_gateway: url.clone(),
                retry_times: 5,
                retry_interval: tokio::time::Duration::from_secs(30),
                identify,
            };
            let mut conn = TungsteniteConnection::new(connect_option, event_tx.clone());
            conn.connect()
                .await
                .map_err(BotBuildError::WsConnectError)?;
        }

        Ok(Bot {
            api_client: Arc::new(api_client),
            event_tx: Arc::new(event_tx),
            cache: BotCache::default(),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            // dispacher: Arc::new(RwLock::new(EventDispatcher::default())),
        })
    }
}

#[derive(Debug)]
pub enum BotError {
    ApiError(reqwest::Error),
    BadRequest(crate::api::ResponseFail),
}

/// Handle Events
impl Bot {
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ClientEvent> {
        self.event_tx.subscribe()
    }
    pub async fn register_boxed_handler(&self, name: String, handler: Box<dyn Handler>) {
        let mut rx = self.subscribe();
        let ctx = Arc::new(self.clone());
        let task = tokio::spawn(async move {
            while let Ok(seq_evt) = rx.recv().await {
                let result = handler.handle(seq_evt, ctx.clone());
                drop(result);
            }
        });
        if let Some(jh) = self.handlers.write().await.insert(name, task) {
            jh.abort();
        }
    }
    pub async fn register_handler<H: Handler + 'static>(
        &self,
        name: impl Into<String>,
        handler: H,
    ) {
        self.register_boxed_handler(name.into(), Box::new(handler))
            .await;
    }
    pub async fn unregister_handler(&self, name: &str) {
        if let Some(jh) = self.handlers.write().await.remove(name) {
            jh.abort();
        }
    }
}
