use crate::websocket::{Identify, Resume};

// pub mod awc_client;
pub mod reqwest_client;
pub mod tungstenite_client;
pub mod actix_ws_client;

#[derive(Debug, Clone)]
pub enum ConnectType {
    New(Identify),
    Reconnect(Resume),
}

#[derive(Debug, Clone)]
pub struct ConnectOption {
    pub wss_gateway: String,
    pub connect_type: ConnectType,
}
