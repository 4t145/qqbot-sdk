use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    RwLock,
    oneshot::{Receiver, Sender},
};

use crate::model::MessageAudited;
#[derive(Debug)]
pub struct AuditHook {
    pub(crate) tx: Sender<Arc<MessageAudited>>,
}

pub struct AuditHookAwaiter {
    pub(crate) rx: Receiver<Arc<MessageAudited>>,
}
impl AuditHookAwaiter {
    pub async fn await_hook(self) -> crate::Result<Arc<MessageAudited>> {
        self.rx
            .await
            .map_err(|_| crate::Error::unexpected("audit hook await error"))
    }
}

impl AuditHook {
    pub fn new() -> (Self, AuditHookAwaiter) {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        (Self { tx: sender }, AuditHookAwaiter { rx: receiver })
    }
}

#[derive(Clone, Debug, Default)]
pub struct AuditHookPool {
    hooks: Arc<RwLock<HashMap<String, AuditHook>>>,
}
impl AuditHookPool {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn insert(&self, audit_id: String) -> AuditHookAwaiter {
        let (hook, awaiter) = AuditHook::new();
        self.hooks.write().await.insert(audit_id, hook);
        awaiter
    }
    pub async fn remove(&self, audit_id: &str) -> Option<AuditHook> {
        self.hooks.write().await.remove(audit_id)
    }
}
