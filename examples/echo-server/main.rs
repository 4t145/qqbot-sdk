use qqbot_sdk::{
    bot::{Bot, BotConfig, message::MessageBuilder},
    event::{handler::EventHandler, model::Event},
    http::api::reaction::EmojiReactionDescriptor,
    model::{Emoji, RawEmoji},
};

#[tokio::main]
async fn main() {
    async_main().await.unwrap();
}
#[derive(Debug)]
pub struct EchoHandler;

impl EventHandler for EchoHandler {
    fn would_handle(&self, _event: &Event, _bot: &Bot) -> bool {
        true
    }
    async fn handle(&self, event: Event, ctx: &Bot) -> Result<(), qqbot_sdk::Error> {
        match event {
            Event::MessageCreate(m) if !m.author.bot => {
                tracing::info!("message: {:?}", m);
                // 贴猴
                ctx.create_reaction(&EmojiReactionDescriptor {
                    channel_id: m.channel_id,
                    message_id: m.id.clone(),
                    emoji: Emoji::Raw(RawEmoji::猴),
                })
                .await?;
            }
            Event::MessageDelete(m) => {
                tracing::info!("message delete: {:?}", m);
                ctx.send_message_public_channel(
                    m.message.channel_id,
                    &MessageBuilder::default()
                        // .reply_to_id(m.message.id)
                        .content("message delete".to_string().as_str())
                        .build(),
                )
                .await?;
            }
            Event::AtMessageCreate(m) => {
                let channel_id = m.channel_id;
                let sender = &m.author.clone();
                match ctx
                    .send_message(
                        channel_id,
                        &MessageBuilder::default()
                            .content(m.content.as_str())
                            .reply_to(m.as_ref())
                            .build(),
                    )
                    .await
                {
                    Ok(_) => tracing::info!("echo: {:?}", sender),
                    Err(e) => tracing::error!("echo: {:?}, error: {:?}", sender, e),
                };
            }
            other => tracing::info!("event: {:?}", other),
        }
        Ok(())
    }
}
async fn async_main() -> Result<(), qqbot_sdk::Error> {
    // 启动webapi client
    let app_id = std::env::var("APP_ID").unwrap();
    let secret = std::env::var("SECRET").unwrap();
    let bot = Bot::new(BotConfig {
        app_id,
        secret,
        base_url: qqbot_sdk::consts::SANDBOX_DOMAIN.to_string(),
    });
    bot.start_webhook_service(([0, 0, 0, 0], 8080).into())
        .await?;
    tracing::info!("bot: {:?}", bot.about_me().await?);
    bot.fetch_my_guilds().await?;
    tracing::info!("guilds count: {:?}", bot.cache().get_guilds_count().await);
    bot.event_service().spawn_handler("echo", EchoHandler).await;
    
    // wait for ctrl-c
    // bot.await;
    // wait for ctrl-c
    tokio::signal::ctrl_c().await.unwrap();
    bot.stop();
    Ok(())
}
