use std::sync::Arc;

use crate::client::ClientEvent;

use super::{Bot, BotError};
pub trait Handler: std::fmt::Debug + Send + Sync {
    fn handle(&self, event: ClientEvent, ctx: Arc<Bot>) -> Result<(), BotError>;
}
