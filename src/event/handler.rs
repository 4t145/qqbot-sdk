use super::model::Event;
pub use crate::bot::Bot;
use std::sync::Arc;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EventHandlerId(Arc<str>);

impl From<&str> for EventHandlerId {
    fn from(id: &str) -> Self {
        Self(Arc::from(id))
    }
}

impl EventHandlerId {
    pub fn new(id: impl AsRef<str>) -> Self {
        Self(Arc::from(id.as_ref()))
    }
}

pub trait EventHandler<C: Clone = ()>: Send + Sync + 'static {
    fn would_handle(&self, event: &Event, bot: &Bot<C>) -> bool;
    fn handle(&self, event: Event, bot: &Bot<C>) -> impl Future<Output = crate::Result<()>> + Send;
}
