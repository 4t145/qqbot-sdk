//! # design
//! ## recoverale
//! 重新连接
//!

use std::{error::Error, sync::Arc};

use tokio::sync::broadcast;

use crate::{
    model::{MessageAudited, MessageBotRecieved, MessageReaction},
    websocket::{Event, Identify, Resume},
};

// pub mod awc_client;
pub mod reqwest_client;
pub mod tungstenite_client;
// pub mod actix_ws_client;

#[derive(Debug, Clone)]
pub enum ConnectType {
    New(Identify),
    Reconnect(Resume),
}

impl ConnectType {
    pub fn get_token(&self) -> &str {
        match self {
            ConnectType::New(Identify { token, .. }) => token,
            ConnectType::Reconnect(Resume { token, .. }) => token,
        }
    }
}
#[derive(Debug, Clone)]
pub struct ConnectConfig {
    pub wss_gateway: String,
    pub identify: Identify,
    pub retry_times: usize,
    pub retry_interval: tokio::time::Duration,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ConnectionState {
    Connecting = 0,
    Connected = 1,
    Reconnecting = 2,
    #[default]
    Disconnected = 3,
    Guaranteed = 4,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Connecting => write!(f, "Connecting"),
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Reconnecting => write!(f, "Reconnecting"),
            ConnectionState::Disconnected => write!(f, "Disconnected"),
            ConnectionState::Guaranteed => write!(f, "Guaranteed"),
        }
    }
}

impl From<ConnectionState> for u8 {
    fn from(val: ConnectionState) -> Self {
        val as u8
    }
}

impl From<u8> for ConnectionState {
    fn from(val: u8) -> Self {
        match val {
            0 => ConnectionState::Connecting,
            1 => ConnectionState::Connected,
            2 => ConnectionState::Reconnecting,
            3 => ConnectionState::Disconnected,
            4 => ConnectionState::Guaranteed,
            _ => ConnectionState::Guaranteed,
        }
    }
}

#[async_trait::async_trait]
pub trait Connection {
    type Error: Error + Send + Sync + 'static;
    fn new(connect_config: ConnectConfig, event_sender: broadcast::Sender<ClientEvent>) -> Self;
    fn get_state(&self) -> ConnectionState;
    fn get_config(&self) -> &ConnectConfig;
    fn confict_state_err(state: ConnectionState, expected: ConnectionState) -> Self::Error;
    async fn connect(&mut self) -> Result<(), Self::Error> {
        let state = self.get_state();
        match state {
            ConnectionState::Connecting => {
                log::warn!("Trying to connect while connecting");
                Err(Self::confict_state_err(state, ConnectionState::Connecting))
            }
            ConnectionState::Connected => {
                log::warn!("Already connected");
                Ok(())
            }
            ConnectionState::Reconnecting => {
                log::warn!("Trying to connect while reconnecting");
                Err(Self::confict_state_err(state, ConnectionState::Disconnected))
            }
            ConnectionState::Disconnected => {
                log::info!("Start Connecting");
                self.connect_inner().await
            }
            ConnectionState::Guaranteed => {
                log::warn!("Trying to connect while guaranteed");
                Err(Self::confict_state_err(state, ConnectionState::Disconnected))
            }
        }
    }
    async fn connect_inner(&mut self) -> Result<(), Self::Error>;
    async fn reconnect(&mut self) -> Result<(), Self::Error> {
        let state = self.get_state();
        match state {
            ConnectionState::Connecting => {
                log::warn!("Trying to reconnect while connecting");
                Err(Self::confict_state_err(state, ConnectionState::Disconnected))
            }
            ConnectionState::Connected => {
                log::warn!("Already connected");
                Ok(())
            }
            ConnectionState::Reconnecting => {
                log::warn!("Trying to reconnect while reconnecting");
                Err(Self::confict_state_err(state, ConnectionState::Disconnected))
            }
            ConnectionState::Disconnected => {
                log::info!("Start Reconnecting");
                self.reconnect_inner().await
            }
            ConnectionState::Guaranteed => {
                log::warn!("Trying to connect while guaranteed");
                Err(Self::confict_state_err(state, ConnectionState::Disconnected))
            }
        }
    }
    async fn reconnect_inner(&mut self) -> Result<(), Self::Error>;

    async fn wait_disconect(&mut self) -> Result<(), Self::Error> {
        let state = self.get_state();
        match state {
            ConnectionState::Connecting => {
                log::warn!("Trying to wait_disconect while connecting");
                Err(Self::confict_state_err(state, ConnectionState::Disconnected))
            }
            ConnectionState::Connected => {
                log::info!("Start wait_disconect");
                self.wait_disconect_inner().await
            }
            ConnectionState::Reconnecting => {
                log::warn!("Trying to wait_disconect while reconnecting");
                Err(Self::confict_state_err(state, ConnectionState::Disconnected))
            }
            ConnectionState::Disconnected => {
                log::warn!("Already disconnected");
                Ok(())
            }
            ConnectionState::Guaranteed => {
                log::warn!("Trying to connect while guaranteed");
                Err(Self::confict_state_err(state, ConnectionState::Disconnected))
            }
        }
    }

    async fn wait_disconect_inner(&mut self) -> Result<(), Self::Error>;

}

// pub type SeqEvent = (Event, u32);

#[derive(Debug, Clone)]
pub enum ClientEvent {
    AtMessageCreate(Arc<MessageBotRecieved>),
    MessageAuditPass(Arc<MessageAudited>),
    MessageAuditReject(Arc<MessageAudited>),
    MessageReactionAdd(Arc<MessageReaction>),
    MessageReactionRemove(Arc<MessageReaction>),
}

impl TryFrom<Event> for ClientEvent {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        macro_rules! map_event {
            ($val:expr => $(|$event:ident)*) => {
                match $val {
                    $(Event::$event(msg) => {Ok(ClientEvent::$event(Arc::new(*msg)))})*
                    _ => Err(())
                }
            };
        }
        map_event! {
            event =>
                | AtMessageCreate
                | MessageAuditPass
                | MessageAuditReject
                | MessageReactionAdd
                | MessageReactionRemove
        }
    }
}
