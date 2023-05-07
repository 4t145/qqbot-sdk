use std::{any::Any, collections::HashMap, pin::Pin, process::Output};

use crate::websocket::Event;
use futures_util::Future;
use std::fmt::Debug;

use super::Bot;

pub trait Filter<T: ?Sized> {
    type Output: ?Sized;
    fn filter(&self, message: &T) -> Option<Box<Self::Output>>;
}

// pub trait Handler<F>: Debug
// where
//     F: Filter + ?Sized,
// {
//     fn handle(
//         &self,
//         message: &F::Output,
//         seq: u32,
//         ctx: &Bot,
//     ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
// }

// pub trait Dispatcher<T> {
//     type Message: ?Sized;
//     fn filter(&self, message: &T) -> Option<Self::Message>;
//     fn dispatch(&self, message: &Self::Message);
// }


// pub struct Dispatcher<T> {
//     children: HashMap<String, >,
// }