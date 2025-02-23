#![warn(clippy::unwrap_used)]
pub(crate) mod utils;

/// 机器人
// pub mod _bot;
pub mod bot;
/// 静态量，常量
pub mod consts;
/// 错误处理
mod error;
/// 用于处理 HTTP， 频道主动api
pub mod http;
/// 数据结构
pub mod model;

pub use error::{Error, Result};

pub mod event;
