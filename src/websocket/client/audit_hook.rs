use std::{collections::btree_map::BTreeMap, fmt::Debug};

use tokio::sync::{oneshot, Mutex};

use crate::model::MessageAudited;

pub struct AuditHook {
    expire_time: tokio::time::Instant,
    sender: oneshot::Sender<AuditResult>,
}
#[repr(transparent)]
pub struct AuditTask(oneshot::Receiver<AuditResult>);

impl AuditTask {
    #[inline]
    pub async fn await_result(self) -> AuditResult {
        // if sender is cleaned, it means the audit is timeout
        self.0.await.unwrap_or(AuditResult::Timeout)
    }
}

pub enum AuditResult {
    Pass(MessageAudited),
    Reject(MessageAudited),
    Timeout,
}

pub struct AuditHookPool {
    expire: tokio::time::Duration,
    pool: Mutex<BTreeMap<String, AuditHook>>,
}

impl Debug for AuditHookPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuditHookPool")
            .field("expire", &self.expire)
            .finish()
    }
}
impl AuditHookPool {
    pub fn new(expire: tokio::time::Duration) -> Self {
        Self {
            expire,
            pool: Mutex::new(BTreeMap::new()),
        }
    }

    pub async fn resolve(&self, key: &str, result: AuditResult) {
        let mut pool = self.pool.lock().await;
        if let Some(hook) = pool.remove(key) {
            // we don't have responsibility to handle the result, hook create side should care about it
            hook.sender.send(result).unwrap_or_default();
        }
    }

    pub async fn create(&self, key: String) -> AuditTask {
        let (sender, receiver) = oneshot::channel();
        let hook = AuditHook {
            expire_time: tokio::time::Instant::now() + self.expire,
            sender,
        };
        let mut pool = self.pool.lock().await;
        pool.insert(key, hook);
        AuditTask(receiver)
    }

    pub async fn clean(&self) {
        let mut pool = self.pool.lock().await;
        let now = tokio::time::Instant::now();
        pool.retain(|_, v| v.expire_time > now);
    }
}
