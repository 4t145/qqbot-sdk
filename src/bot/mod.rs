mod dispacher;
mod handler;
mod message;
mod methods;
mod shard;
mod user;

use crate::{
    http::api::{
        websocket::{Gateway, GatewayBot},
        Authority,
    },
    http::client::reqwest_client::ApiClient,
    model::Guild,
    websocket::Identify,
    websocket::{audit_hook::AuditHookPool, ClientEvent, ConnectConfig, Connection},
};
use futures_util::Future;
pub use message::*;
pub use shard::Shards;
use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    ops::Deref,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::{sync::RwLock, task::JoinHandle};
pub use user::*;

pub use self::handler::Handler;

// use self::dispacher::{EventDispatcher};

/// # Bot
/// ## Clone
/// 可以随意克隆bot，内部全部有arc包裹, 且不提供可变性
#[derive(Debug)]
pub struct BotInner<
    C: crate::websocket::Connection = crate::websocket::tungstenite_client::TungsteniteConnection,
> {
    api_client: ApiClient,
    event_tx: tokio::sync::broadcast::Sender<ClientEvent>,
    cache: BotCache,
    handlers: RwLock<HashMap<String, JoinHandle<()>>>, // dispacher: Arc<RwLock<EventDispatcher>>,
    conn_tasks: Vec<JoinHandle<Result<(), C::Error>>>,
    audit_hook_pool: Arc<AuditHookPool>,
}

impl<C: Connection> BotInner<C> {
    fn get_conn_task_status(&self) -> Vec<bool> {
        self.conn_tasks.iter().map(|h| !h.is_finished()).collect()
    }
    fn is_conn_health(&self) -> bool {
        if self.conn_tasks.is_empty() {
            return false;
        }
        self.get_conn_task_status().iter().all(|&b| b)
    }
}

impl<C: Connection> Drop for BotInner<C> {
    fn drop(&mut self) {
        tokio::task::block_in_place(|| {
            log::debug!("Bot is dropping, aborting all tasks");
            for (_, h) in self.handlers.blocking_write().drain() {
                h.abort();
            }
            for h in &self.conn_tasks {
                h.abort()
            }
        });
    }
}

#[derive(Debug)]
pub struct Bot<
    C: crate::websocket::Connection = crate::websocket::tungstenite_client::TungsteniteConnection,
>(Arc<BotInner<C>>);

impl Clone for Bot {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Deref for Bot {
    type Target = BotInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
    WsConnectError(crate::websocket::tungstenite_client::TungsteniteConnectionError),
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

    pub async fn start<C: Connection + Send>(mut self) -> Result<Bot<C>, BotError> {
        let audit_expire = tokio::time::Duration::from_secs(30);
        let hook_pool = Arc::new(AuditHookPool::new(audit_expire));
        let auth = self.authority.ok_or(BotBuildError::NoAuthority)?;
        let token = auth.token();
        let api_client = ApiClient::new(auth);
        let url = if self.auto_shard {
            let response = api_client
                .send::<GatewayBot>(&())
                .await
                .map_err(BotBuildError::ApiError)?
                .as_result()?;
            self.shards = Some(Shards::new_all(response.shards));
            response.url
        } else {
            api_client
                .send::<Gateway>(&())
                .await
                .map_err(BotBuildError::ApiError)?
                .as_result()?
                .url
        };
        log::info!("ws gate url: {}", url);
        let (event_tx, _event_rx) = tokio::sync::broadcast::channel(128);
        let mut task_handles = vec![];
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
                let conn_task = connect_option
                    .start_connection_with_ctrl_c::<C>(event_tx.clone(), hook_pool.clone());
                task_handles.push(conn_task);
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
                wss_gateway: url,
                retry_times: 5,
                retry_interval: tokio::time::Duration::from_secs(30),
                identify,
            };
            let _conn_task = connect_option
                .start_connection_with_ctrl_c::<C>(event_tx.clone(), hook_pool.clone());
            task_handles.push(_conn_task);
        }

        Ok(Bot(Arc::new(BotInner {
            api_client,
            event_tx,
            cache: BotCache::default(),
            handlers: RwLock::new(HashMap::new()),
            audit_hook_pool: hook_pool,
            conn_tasks: task_handles,
        })))
    }
}

#[derive(Debug)]
pub enum BotError {
    ApiError(reqwest::Error),
    BadRequest(crate::http::api::ResponseFail),
    BuildError(BotBuildError),
    AuditTimeout,
}

impl Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotError::ApiError(e) => write!(f, "api error: {}", e),
            BotError::BadRequest(e) => write!(f, "bad request: {}", e),
            BotError::BuildError(e) => write!(f, "build error: {:?}", e),
            BotError::AuditTimeout => write!(f, "audit timeout"),
        }
    }
}

impl Error for BotError {}

impl From<reqwest::Error> for BotError {
    fn from(val: reqwest::Error) -> Self {
        BotError::ApiError(val)
    }
}

impl From<crate::http::api::ResponseFail> for BotError {
    fn from(val: crate::http::api::ResponseFail) -> Self {
        BotError::BadRequest(val)
    }
}

impl From<BotBuildError> for BotError {
    fn from(val: BotBuildError) -> Self {
        BotError::BuildError(val)
    }
}
/// error and recover
impl Bot {
    pub fn is_conn_health(&self) -> bool {
        self.0.is_conn_health()
    }
}

impl Future for Bot {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let health = self.is_conn_health();
        cx.waker().wake_by_ref();

        dbg!("polling bot", health);
        if health {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
