use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, atomic::AtomicBool},
};

use audit_hook_pool::AuditHookPool;
use futures_util::{Stream, StreamExt};
pub mod implement;
pub mod model;
use crate::bot::BotRef;
use handler::EventHandlerId;
use model::Event;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
pub mod audit_hook_pool;
pub mod handler;
// pub trait EventService: Stream<Item = Event> {

// }

pub trait EventStreamProvider:
    Stream<Item = Event> + Unpin + Sized + Send + Sync + 'static
{
    fn name(&self) -> Cow<'static, str>;
}

pub struct EventService<C: Clone = ()> {
    running: Arc<AtomicBool>,
    audit_hook_pool: AuditHookPool,
    handlers: Arc<RwLock<HashMap<EventHandlerId, CancellationToken>>>,
    event_dispatch_channel: tokio::sync::broadcast::Sender<Event>,
    bot: BotRef<C>,
    ct: CancellationToken,
}

impl<C: Clone> EventService<C>
where
    C: Clone + Send + 'static,
{
    const DEFAULT_CHANNEL_SIZE: usize = 4096;
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }
    pub fn new(bot: BotRef<C>) -> Self {
        let (event_dispatch_channel, _event_subscribe) =
            tokio::sync::broadcast::channel(Self::DEFAULT_CHANNEL_SIZE);
        Self {
            running: Arc::new(AtomicBool::new(false)),
            audit_hook_pool: AuditHookPool::new(),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_dispatch_channel,
            bot,
            ct: CancellationToken::new(),
        }
    }
    pub(crate) fn spawn<P: EventStreamProvider>(&self, provider: P) -> crate::Result<()> {
        let Some(bot) = self.bot.upgrade() else {
            return Err(crate::Error::unexpected("bot dropped"));
        };
        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(crate::Error::unexpected("event service is running"));
        }
        {
            let ct = bot.ct.child_token();
            let event_dispatch_channel = self.event_dispatch_channel.clone();
            let audit_hook_pool = self.audit_hook_pool.clone();
            let running = self.running.clone();
            tokio::spawn(async move {
                let provider_name = provider.name().as_ref().to_string();
                let mut stream = provider;
                loop {
                    let event = tokio::select! {
                        _ = ct.cancelled() => {
                            break;
                        },
                        evt = stream.next() => {
                            if let Some(evt) = evt {
                                evt
                            } else {
                                break;
                            }
                        }
                    };
                    match &event {
                        Event::MessageAuditPass(message_audited) => {
                            if let Some(hook) =
                                audit_hook_pool.remove(&message_audited.audit_id).await
                            {
                                let _send_result = hook.tx.send(message_audited.clone());
                            }
                        }
                        Event::MessageAuditReject(message_audited) => {
                            if let Some(hook) =
                                audit_hook_pool.remove(&message_audited.audit_id).await
                            {
                                let _send_result = hook.tx.send(message_audited.clone());
                            }
                        }
                        _ => {}
                    }
                    let Ok(received_count) =
                        event_dispatch_channel.send(event).inspect_err(|_err| {
                            tracing::warn!(provider_name, "event dispatch channel send error");
                        })
                    else {
                        break;
                    };
                    tracing::debug!(provider_name, received_count, "event dispatched");
                }
                tracing::info!(provider_name, "event stream ended");
                running.store(false, std::sync::atomic::Ordering::SeqCst);
            });
        };
        Ok(())
    }
    pub async fn register_audit_hook(
        &self,
        message_id: String,
    ) -> audit_hook_pool::AuditHookAwaiter {
        self.audit_hook_pool.insert(message_id).await
    }

    pub async fn spawn_handler<H: handler::EventHandler<C>>(
        &self,
        id: impl Into<EventHandlerId>,
        handler: H,
    ) {
        let id = id.into();
        let ct = self.ct.child_token();
        let mut rx = self.event_dispatch_channel.subscribe();
        let bot_ref = self.bot.clone();
        self.handlers.write().await.insert(id.clone(), ct.clone());
        tokio::spawn(async move {
            loop {
                let event = tokio::select! {
                    _ = ct.cancelled() => {
                        break;
                    },
                    evt = rx.recv()  => {
                        if let Ok(evt) = evt {
                            evt
                        } else {
                            break;
                        }
                    }
                };
                let Some(bot) = bot_ref.upgrade() else {
                    break;
                };
                if handler.would_handle(&event, &bot) {
                    let handle_result = handler.handle(event, &bot).await;
                    if let Err(err) = handle_result {
                        tracing::warn!("handler error: {:?}", err);
                    }
                }
                if ct.is_cancelled() {
                    break;
                }
            }
            if let Some(bot) = bot_ref.upgrade() {
                bot.event_service.handlers.write().await.remove(&id);
            }
        });
    }

    pub async fn shutdown_handler(&self, id: &EventHandlerId) {
        if let Some(ct) = self.handlers.write().await.remove(id) {
            ct.cancel();
        }
    }

    pub async fn shutdown(&self) {
        self.ct.cancel();
    }
}
