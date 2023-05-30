# QQBOT-SDK
qq频道机器人sdk

## 使用例
```rust
use std::sync::Arc;

use qqbot_sdk::{
    api::Authority,
    bot::{Bot, BotBuilder, BotError, Handler, MessageBuilder},
    client::tungstenite_client::SeqEvent,
    websocket::{Intends, Event},
};

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug,qqbot_sdk=DEBUG");
    env_logger::builder().is_test(true).try_init().unwrap();
    async_main().await.unwrap();
}

#[derive(Debug)]
pub struct EchoHandler;

impl Handler for EchoHandler {
    fn handle(&self, event: SeqEvent, ctx: Arc<Bot>) -> Result<(), BotError> {
        tokio::spawn(async move {
            match event.0 {
                Event::AtMessageCreate(m) => {
                    let channel_id = m.channel_id;
                    ctx.post_message(
                        channel_id,
                        &MessageBuilder::default()
                            .content(m.content.as_str())
                            .reply_to(m.as_ref())
                            .build()
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                }
                other => log::info!("event: {:?}", other),
            }
        });
        Ok(())
    }
}
async fn async_main() -> Result<(), BotError> {
    // 启动webapi client
    let auth = Authority::Bot {
        app_id: &std::env::var("APP_ID").unwrap(),
        token: &std::env::var("TOKEN").unwrap(),
    };
    let bot = BotBuilder::default()
        .auth(auth)
        .intents(Intends::PUBLIC_GUILD_MESSAGES | Intends::GUILD_MESSAGE_REACTIONS)
        .build()
        .await
        .unwrap();
    log::info!("bot: {:?}", bot.about_me().await?);
    bot.fetch_my_guilds().await?;
    log::info!("guilds count: {:?}", bot.cache().get_guilds_count().await);
    bot.register_handler("echo", EchoHandler).await;
    // wait for ctrl-c
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
```