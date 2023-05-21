use std::{ collections::HashMap, hash::Hash, sync::Arc};

pub mod channel;
pub mod guild;

pub trait FilterContext {
    type Key: Hash + Eq + Send + Sync + 'static;
    type Message;
    fn add<F>(&self, key: Self::Key, filter: F)
    where
        F: Filter<Context = Self> + Send + Sync + 'static,
        Self: Sized;
    fn remove(&self, key: &Self::Key);
}

type SubFilters<K, T> =
    Arc<tokio::sync::RwLock<HashMap<K, Box<dyn Filter<Context = T> + Send + Sync + 'static>>>>;
pub trait Filter {
    type Context: FilterContext;
    fn handle(&self, message: <Self::Context as FilterContext>::Message);
}
