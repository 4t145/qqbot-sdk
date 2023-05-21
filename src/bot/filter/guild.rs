

use super::{channel::ChannelFilter, *};
use crate::{
    bot::Bot,
    client::tungstenite_client::SeqEvent,
    model::{ChannelId, Guild, MessageBotRecieved, MessageReaction},
};

pub struct GuildFilter {
    guild_id: u64,
    channels: SubFilters<ChannelId, Self>,
}

pub enum GuildEvent {
    Message(Box<MessageBotRecieved>),
    ReactionAdd(Box<MessageReaction>),
    ReactionRemove(Box<MessageReaction>),
}

impl GuildEvent {
    pub fn get_channel_id(&self) -> ChannelId {
        match self {
            Self::Message(m) => m.channel_id,
            Self::ReactionAdd(m) => m.channel_id,
            Self::ReactionRemove(m) => m.channel_id,
        }
    }
}

impl Filter for GuildFilter {
    type Context = Bot;

    fn handle(&self, message: <Self::Context as FilterContext>::Message)
    where
        Self: Sized,
    {
        let channels = self.channels.clone();
        tokio::spawn(async move {
            match &message.0 {
                crate::websocket::Event::AtMessageCreate(m) => {
                    if let Some(channel) = channels.read().await.get(&m.channel_id) {
                        channel.handle(GuildEvent::Message(m.clone()).into());
                    }
                }
                crate::websocket::Event::MessgaeReactionAdd(m) => {
                    if let Some(channel) = channels.read().await.get(&m.channel_id) {
                        channel.handle(GuildEvent::ReactionAdd(m.clone()).into());
                    }
                }
                crate::websocket::Event::MessgaeReactionRemove(m) => {
                    if let Some(channel) = channels.read().await.get(&m.channel_id) {
                        channel.handle(GuildEvent::ReactionRemove(m.clone()).into());
                    }
                }
                _ => {
                    log::trace!("GuildFilter::handle: {:?}", message.0)
                }
            }
        });
    }
}

impl FilterContext for GuildFilter {
    type Message = Arc<GuildEvent>;
    type Key = ChannelId;
    fn add<F>(&self, key: ChannelId, filter: F)
    where
        F: Filter<Context = Self> + Send + Sync + 'static,
        Self: Sized,
    {
        let channels = self.channels.clone();
        tokio::spawn(async move {
            channels
                .write()
                .await
                .insert(key, Box::new(filter));
        });
    }

    fn remove(&self, key: &ChannelId) {
        let channels = self.channels.clone();
        let key = *key;
        tokio::spawn(async move {
            channels.write().await.remove(&key);
        });
    }
}
