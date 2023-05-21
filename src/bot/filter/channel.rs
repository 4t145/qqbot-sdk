use std::sync::Arc;

use super::{Filter, FilterContext, guild::{GuildFilter, GuildEvent}, SubFilters};

#[derive(Default)]
pub struct ChannelFilter {
    pub channel_id: u64,
    pub handlers: SubFilters<String, Self>,
    pub remark: String,
}

impl ChannelFilter {
    pub fn new(channel_id: u64) -> Self {
        Self { 
            channel_id,
            ..Default::default()
        }
    }
}

impl Filter for ChannelFilter {
    type Context = GuildFilter;

    fn handle(&self, message: <Self::Context as FilterContext>::Message)
    where
    Self: Sized
     {
        let handlers = self.handlers.clone();
        tokio::spawn(async move {
            let handlers = handlers.read().await;
            for (_, filter) in handlers.iter() {
                filter.handle(message.clone());
            }
        });
    }
}

impl FilterContext for ChannelFilter {
    type Key = String;
    type Message = Arc<GuildEvent>;
    fn add<F>(&self, key: Self::Key, filter: F)
    where
        F: Filter<Context = Self> + Send + Sync + 'static,
        Self: Sized,
    {
        let handlers = self.handlers.clone();
        tokio::spawn(async move {
            handlers.write().await.insert(key, Box::new(filter));
        });
    }

    fn remove(&self, key: &Self::Key) {
        let handlers = self.handlers.clone();
        let key = key.clone();
        tokio::spawn(async move {
            handlers.write().await.remove(&key);
        });
    }
}