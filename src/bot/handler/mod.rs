use std::sync::Arc;

use crate::client::ClientEvent;

use super::{Bot, BotError};
pub trait Handler: std::fmt::Debug + Send + Sync {
    fn handle(&self, event: ClientEvent, ctx: Arc<Bot>) -> Result<(), BotError>;
}

/// Handle Events
impl Bot {
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ClientEvent> {
        self.event_tx.subscribe()
    }
    pub async fn register_boxed_handler(&self, name: String, handler: Box<dyn Handler>) {
        let mut rx = self.subscribe();
        let ctx = Arc::new(Bot::clone(self));
        let task = tokio::spawn(async move {
            while let Ok(seq_evt) = rx.recv().await {
                let result = handler.handle(seq_evt, ctx.clone());
                drop(result);
            }
        });
        if let Some(jh) = self.handlers.write().await.insert(name, task) {
            jh.abort();
        }
    }
    pub async fn register_handler<H: Handler + 'static>(
        &self,
        name: impl Into<String>,
        handler: H,
    ) {
        self.register_boxed_handler(name.into(), Box::new(handler))
            .await;
    }
    pub async fn unregister_handler(&self, name: &str) {
        if let Some(jh) = self.handlers.write().await.remove(name) {
            jh.abort();
        }
    }
    pub async fn unregister_all_handlers(&self) {
        let mut handlers = self.handlers.write().await;
        for (_, jh) in handlers.drain() {
            jh.abort();
        }
    }
}
