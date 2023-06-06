use std::sync::Arc;

use qqbot_sdk::{
    bot::{Bot, BotBuilder, BotError, Handler, MessageBuilder},
    http::api::{reaction::EmojiReactionDescriptor, Authority},
    model::{Emoji, RawEmoji},
    websocket::ClientEvent,
    websocket::Intends,
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
    fn handle(&self, event: ClientEvent, ctx: Arc<Bot>) -> Result<(), BotError> {
        tokio::spawn(async move {
            match event {
                ClientEvent::MessageCreate(m) if !m.author.bot => {
                    log::info!("message: {:?}", m);
                    // 贴猴
                    ctx.create_reaction(&EmojiReactionDescriptor {
                        channel_id: m.channel_id,
                        message_id: m.id,
                        emoji: Emoji::Raw(RawEmoji::猴),
                    })
                    .await
                    .unwrap_or_default();
                }
                ClientEvent::MessageDelete(m) => {
                    log::info!("message delete: {:?}", m);
                    ctx.send_message_public(
                        m.message.channel_id,
                        &MessageBuilder::default()
                            // .reply_to_id(m.message.id)
                            .content("message delete".to_string().as_str())
                            .build()
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                }
                ClientEvent::AtMessageCreate(m) => {
                    let channel_id = m.channel_id;
                    let sender = &m.author.clone();
                    match ctx
                        .send_message(
                            channel_id,
                            &MessageBuilder::default()
                                .content(m.content.as_str())
                                .reply_to(m.as_ref())
                                .build()
                                .unwrap(),
                        )
                        .await
                    {
                        Ok(_) => log::info!("echo: {:?}", sender),
                        Err(e) => log::error!("echo: {:?}, error: {:?}", sender, e),
                    };
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
        .intents(
            Intends::PUBLIC_GUILD_MESSAGES
                | Intends::GUILD_MESSAGE_REACTIONS
                | Intends::GUILD_MESSAGES,
        )
        .start()
        .await
        .unwrap();
    log::info!("bot: {:?}", bot.about_me().await?);
    bot.fetch_my_guilds().await?;
    log::info!("guilds count: {:?}", bot.cache().get_guilds_count().await);
    bot.register_handler("echo", EchoHandler).await;
    // wait for ctrl-c
    // bot.await;
    // wait for ctrl-c
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
