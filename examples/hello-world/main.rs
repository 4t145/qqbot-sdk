use qqbot_sdk::{
    api::{websocket::Gateway, Authority},
    bot::{Bot, BotBuilder, Channel, MessageBuilder, BotError},
    client::{reqwest_client::*, tungstenite_client::*},
    websocket::{Identify, Intends},
};

use reqwest::Error;

#[tokio::main]
async fn main() {
    async_main().await.unwrap();
}

async fn async_main() -> Result<(), BotError> {
    std::env::set_var("RUST_LOG", "debug,qqbot_sdk=DEBUG");
    env_logger::builder().is_test(true).try_init().unwrap();
    log::info!("Hello, world!");
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
    let me = bot.about_me().await?;
    log::info!("bot: {:?}", me);
    bot.fetch_my_guilds().await?;
    log::info!("guilds count: {:?}", bot.cache().get_guilds_count().await);
    while let Ok((e, seq)) = bot.subscribe().recv().await {
        match e {
            qqbot_sdk::websocket::Event::AtMessageCreate(m) => {
                let channel_id = m.channel_id;
                bot.post_message(
                    &Channel { channel_id },
                    &MessageBuilder::default()
                        .content(m.content.as_str())
                        .reply_to(m.as_ref())
                        .build()
                        .unwrap(),
                )
                .await
                .unwrap();
            },
            other => log::info!("event: {:?}", other),
        }
    }
    Ok(())
}
