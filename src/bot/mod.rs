
mod dispacher;
mod message;
mod user;
mod filter;

pub use message::*;
use tokio::{sync::RwLock, task::JoinHandle};
pub use user::*;

use crate::{
    api::{
        guild::{GetGuild, GetGuildRequest},
        message::{PostMessage, PostMessageRequest},
        reaction::{
            DeleteEmojiReaction, EmojiReactionDescriptor, GetEmojiReactionUserList,
            GetEmojiReactionUserListRequest, SendEmojiReaction,
        },
        user::GetMe,
        websocket::Gateway,
        Authority,
    },
    client::{
        reqwest_client::ApiClient,
        tungstenite_client::{ConnectOption, ConnectType, WsClient, SeqEvent},
    },
    model::{Guild, MessageSend, User, GuildId},
    websocket::{Event, Identify},
};
use std::{collections::HashMap, sync::Arc};

use self::filter::{channel::ChannelFilter, FilterContext};

// use self::dispacher::{EventDispatcher};

/// # Bot
/// ## Clone
/// 可以随意克隆bot，内部全部有arc包裹, 且不提供可变性
#[derive(Debug, Clone)]
pub struct Bot {
    api_client: Arc<ApiClient>,
    ws_client: Arc<WsClient>,
    cache: BotCache,
    // dispacher: Arc<RwLock<EventDispatcher>>,
    filters: Arc<RwLock<HashMap<GuildId, JoinHandle<()>>>>
}

#[derive(Debug, Clone, Default)]
pub struct BotCache {
    guilds: Arc<RwLock<HashMap<u64, Guild>>>,
    users: Arc<RwLock<HashMap<u64, User>>>,
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
#[derive(Debug, Clone)]
pub struct BotBuilder<'a> {
    authority: Option<Authority<'a>>,
    shard: Option<(u32, u32)>,
    intents: u32,
}

#[derive(Debug)]
pub enum BotBuildError {
    NoAuthority,
    ApiError(reqwest::Error),
    WsConnectError(crate::client::tungstenite_client::ConnectError),
}
impl<'a> Default for BotBuilder<'a> {
    fn default() -> Self {
        Self {
            authority: None,
            shard: Some((0, 1)),
            intents: 0,
        }
    }
}
impl<'a> BotBuilder<'a> {
    pub fn auth(self, authority: Authority<'a>) -> Self {
        Self {
            authority: Some(authority),
            ..self
        }
    }
    pub fn shard(self, shard: u32, total: u32) -> Self {
        Self {
            shard: Some((shard, total)),
            ..self
        }
    }
    pub fn intents(self, intents: u32) -> Self {
        Self {
            intents: self.intents | intents,
            ..self
        }
    }
    pub async fn build(self) -> Result<Bot, BotBuildError> {
        let auth = self.authority.ok_or(BotBuildError::NoAuthority)?;
        let token = auth.token();
        let api_client = ApiClient::new(auth);
        let url = api_client
            .send::<Gateway>(&())
            .await
            .map_err(BotBuildError::ApiError)?
            .as_result()
            .unwrap()
            .url;
        log::info!("ws gate url: {}", url);
        // 启动websocket client
        let identify = Identify {
            token,
            intents: self.intents,
            shard: self.shard.map(|s| [s.0, s.1]),
            properties: std::collections::HashMap::new(),
        };

        // ws连接设置
        let connect_option = ConnectOption {
            wss_gateway: url,
            connect_type: ConnectType::New(identify),
        };

        // ws连接
        let ws_connect = connect_option
            .connect_tokio()
            .await
            .map_err(BotBuildError::WsConnectError)?;

        // ws启动客户端
        let ws_client = ws_connect.luanch_client().await;

        Ok(Bot {
            api_client: Arc::new(api_client),
            ws_client: Arc::new(ws_client),
            cache: BotCache::default(),
            filters: Arc::new(RwLock::new(HashMap::new()))
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
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<(Event, u32)> {
        self.ws_client.subscribe_event()
    }
    // pub fn start_dispatch(&self) {
    //     let dispacher = self.dispacher.clone();
    //     let bot = self.clone();
    //     let mut rx = self.subscribe();
    //     tokio::spawn(async move {
    //         while let Ok((event, seq)) = rx.recv().await {
    //             let dispacher = dispacher.read().await;
    //             dispacher.dispatch(&event, seq, &bot);
    //         }
    //     });
    // }
}

impl FilterContext for Bot {
    type Message = Arc<SeqEvent>;
    type Key = GuildId;

    fn add<F>(&self, key: Self::Key, filter: F)
    where
        F: filter::Filter<Context = Self>  + Sync + Send + 'static {
        let mut rx = self.ws_client.subscribe_event();
        let task = async move {
            while let Ok(seq_evt) = rx.recv().await {
                let arced = Arc::new(seq_evt);
                filter.handle(arced);
            }
        };
        let handle = tokio::spawn(task);
        let filters = self.filters.clone();
        let register = async move {
            filters.write().await.insert(key, handle);
        };
        tokio::spawn(register);
    }

    fn remove(&self, key: &Self::Key) {
        let filters = self.filters.clone();
        let key = key.to_owned();
        let unregister = async move {
            filters.write().await.remove(&key);
        };
        tokio::spawn(unregister);
    }

}
impl Bot {
    pub fn cache(&self) -> BotCache {
        self.cache.clone()
    }
    pub async fn post_message(
        &self,
        channel_id: u64,
        message: &MessageSend<'_>,
    ) -> Result<crate::model::MessageBotRecieved, BotError> {
        let request = PostMessageRequest::new(channel_id, message);
        self.api_client
            .send::<PostMessage>(&request)
            .await
            .map_err(BotError::ApiError)?
            .as_result()
            .map_err(BotError::BadRequest)
    }

    pub async fn about_me(&self) -> Result<crate::model::User, BotError> {
        self.api_client
            .send::<GetMe>(&())
            .await
            .map_err(BotError::ApiError)?
            .as_result()
            .map_err(BotError::BadRequest)
    }

    pub async fn fetch_my_guilds(&self) -> Result<(), BotError> {
        let mut req = crate::api::user::GetMyGuildsRequest::default();
        loop {
            let guilds = self
                .api_client
                .send::<crate::api::user::GetMyGuilds>(&req)
                .await
                .map_err(BotError::ApiError)?
                .as_result()
                .map_err(BotError::BadRequest)?;
            req.after = guilds.last().map(|x| x.id);
            let len = guilds.len();
            log::debug!("guilds: {:#?}", guilds);
            self.cache.cache_many_guilds(guilds).await;
            if len < 100 {
                return Ok(());
            }
        }
    }

    pub async fn get_guild_from_remote(&self, guild_id: u64) -> Result<Guild, BotError> {
        self.api_client
            .send::<GetGuild>(&GetGuildRequest { guild_id })
            .await
            .map_err(BotError::ApiError)?
            .as_result()
            .map_err(BotError::BadRequest)
    }

    pub async fn create_reaction(
        &self,
        reaction: &EmojiReactionDescriptor,
    ) -> Result<(), BotError> {
        self.api_client
            .send::<SendEmojiReaction>(reaction)
            .await
            .map_err(BotError::ApiError)?
            .as_result()
            .map_err(BotError::BadRequest)?;
        Ok(())
    }

    pub async fn delete_reaction(
        &self,
        reaction: &EmojiReactionDescriptor,
    ) -> Result<(), BotError> {
        self.api_client
            .send::<DeleteEmojiReaction>(reaction)
            .await
            .map_err(BotError::ApiError)?
            .as_result()
            .map_err(BotError::BadRequest)?;
        Ok(())
    }

    pub async fn get_reaction_users(
        &self,
        reaction: &EmojiReactionDescriptor,
    ) -> Result<Vec<User>, BotError> {
        let mut req: GetEmojiReactionUserListRequest =
            GetEmojiReactionUserListRequest::new(reaction);
        let mut collector = vec![];
        loop {
            let resp = self
                .api_client
                .send::<GetEmojiReactionUserList>(&req)
                .await
                .map_err(BotError::ApiError)?
                .as_result()
                .map_err(BotError::BadRequest)?;

            collector.extend(resp.users);
            if let Some(cookie) = resp.cookie {
                req.next(cookie);
            }
            if resp.is_end {
                collector.dedup();
                break Ok(collector);
            }
        }
    }
}
