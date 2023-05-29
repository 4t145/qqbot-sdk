use std::sync::Arc;

use crate::client::tungstenite_client::SeqEvent;

use super::{Bot, BotError};
pub trait Handler: std::fmt::Debug + Send + Sync {
    fn handle(&self, event: SeqEvent, ctx: Arc<Bot>) -> Result<(), BotError>;
}
