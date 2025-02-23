pub mod message;
pub mod methods;
pub mod user;

use std::{
    collections::HashMap,
    net::SocketAddr,
    ops::Deref,
    sync::{Arc, Weak},
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    event::{EventService, implement::webhook::WebHookServiceAppConfig},
    http::client::reqwest_client::ApiClient,
    model::Guild,
};
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BotConfig {
    pub app_id: String,
    pub secret: String,
    pub base_url: String,
}

pub struct BotInner {
    pub(crate) event_service: EventService,
    pub(crate) api_client: ApiClient,
    pub(crate) ct: tokio_util::sync::CancellationToken,
    pub(crate) config: BotConfig,
    pub(crate) cache: BotCache,
}

impl BotInner {}

#[derive(Clone)]
pub struct Bot<C: Clone = ()> {
    inner: Arc<BotInner>,
    context: C,
}
impl<C: Clone> Bot<C> {
    pub fn context(&self) -> &C {
        &self.context
    }
    pub fn reference(&self) -> BotRef<C> {
        BotRef {
            inner: Arc::downgrade(&self.inner),
            context: self.context.clone(),
        }
    }
    pub fn with_context<C2: Clone>(&self, context: C2) -> Bot<C2> {
        Bot {
            inner: self.inner.clone(),
            context,
        }
    }
}
impl Bot {
    pub fn new(config: BotConfig) -> Self {
        let inner = Arc::new_cyclic(|inner| BotInner {
            api_client: ApiClient::new(&config.secret, &config.app_id, &config.base_url),
            event_service: EventService::new(BotRef {
                inner: inner.clone(),
                context: (),
            }),
            ct: tokio_util::sync::CancellationToken::new(),
            config,
            cache: BotCache::default(),
        });

        Self { inner, context: () }
    }
    pub async fn start_webhook_service(&self, bind: SocketAddr) -> crate::Result<()> {
        const DEFAULT_CHANNEL_SIZE: usize = 4096;
        // 16 MB
        const DEFAULT_BODY_SIZE: usize = 16 * 1024 * 1024;
        let service = crate::event::implement::webhook::WebHookService::run(
            WebHookServiceAppConfig {
                bind,
                channel_size: DEFAULT_CHANNEL_SIZE,
                max_body_size: DEFAULT_BODY_SIZE,
            },
            &self.config.secret,
            self.ct.child_token(),
        )
        .await?;
        self.event_service.spawn(service)?;
        Ok(())
    }
    pub fn event_service(&self) -> &EventService {
        &self.inner.event_service
    }
    pub fn config(&self) -> &BotConfig {
        &self.inner.config
    }
    pub fn stop(&self) {
        self.inner.ct.cancel();
    }
}

impl<C: Clone> Deref for Bot<C> {
    type Target = BotInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone)]
pub struct BotRef<C: Clone = ()> {
    inner: Weak<BotInner>,
    context: C,
}

impl<C: Clone> BotRef<C> {
    pub fn upgrade(&self) -> Option<Bot<C>> {
        self.inner.upgrade().map(|inner| Bot {
            inner,
            context: self.context.clone(),
        })
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
