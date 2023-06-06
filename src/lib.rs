#![warn(clippy::unwrap_used)]
pub(crate) mod utils;

/// 机器人
pub mod bot;
/// 错误处理
mod error;
/// 用于处理 HTTP， 频道主动api
pub mod http;
/// 数据结构
pub mod model;
/// 静态量，常量
pub mod statics;
/// 用于处理 websocket， 频道被动api
pub mod websocket;
