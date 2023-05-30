use crate::websocket::{Identify, Resume};

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
pub struct ConnectOption {
    pub wss_gateway: String,
    pub connect_type: ConnectType,
    pub retry_times: usize,
    pub retry_interval: tokio::time::Duration,
}
