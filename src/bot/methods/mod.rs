use crate::{
    http::api::{
        guild::{GetGuild, GetGuildRequest},
        message::{PostMessage, PostMessageRequest},
        reaction::{
            DeleteEmojiReaction, EmojiReactionDescriptor, GetEmojiReactionUserList,
            GetEmojiReactionUserListRequest, SendEmojiReaction,
        },
        user::GetMe,
    },
    model::{Guild, MessageSend, User},
};

use super::*;

impl Bot {
    pub fn cache(&self) -> BotCache {
        self.cache.clone()
    }
    pub async fn send_message_public_channel(
        &self,
        channel_id: u64,
        message: &MessageSend<'_>,
    ) -> Result<crate::model::MessageAudited, crate::Error> {
        let request = PostMessageRequest::new(channel_id, message);
        let resp = self
            .api_client
            .send::<PostMessage>(&request)
            .await?
            .as_result();
        match resp {
            Ok(msg) => {
                // it's impossible to get a message_id here
                // because the message is not audited yet
                Ok(msg.into())
            }
            Err(f) => {
                match f.code {
                    // 审核中
                    304023 | 304024 => {
                        let Some(data) = f.data.clone() else {
                            return Err(crate::Error::unexpected(
                                "audited message should have data",
                            ));
                        };
                        let audit_hook_id = data
                            .get("message_audit")
                            .expect("audited body should have message_audit")
                            .get("audit_id")
                            .expect("message_audit should have audit_id")
                            .as_str()
                            .expect("audit_id should be string")
                            .to_owned();
                        let res = self
                            .event_service
                            .register_audit_hook(audit_hook_id)
                            .await
                            .await_hook()
                            .await?;
                        Ok(res.as_ref().clone())
                    }
                    _ => Err(crate::Error::unexpected(format!(
                        "send message failed: {}",
                        f.message
                    ))),
                }
            }
        }
    }
    pub async fn send_message(
        &self,
        channel_id: u64,
        message: &MessageSend<'_>,
    ) -> Result<crate::model::MessageBotRecieved, crate::Error> {
        let request = PostMessageRequest::new(channel_id, message);
        let resp = self
            .api_client
            .send::<PostMessage>(&request)
            .await?
            .as_result()
            .map_err(crate::Error::context("send_message"))?;
        Ok(resp)
    }

    pub async fn about_me(&self) -> Result<crate::model::User, crate::Error> {
        self.api_client
            .send::<GetMe>(&())
            .await?
            .as_result()
            .map_err(crate::Error::context("about_me"))
    }

    pub async fn fetch_my_guilds(&self) -> Result<(), crate::Error> {
        let mut req = crate::http::api::user::GetMyGuildsRequest::default();
        loop {
            let guilds = self
                .api_client
                .send::<crate::http::api::user::GetMyGuilds>(&req)
                .await?
                .as_result()
                .map_err(crate::Error::context("fetch_my_guilds"))?;
            req.after = guilds.last().map(|x| x.id);
            let len = guilds.len();
            tracing::debug!("guilds: {:#?}", guilds);
            self.cache.cache_many_guilds(guilds).await;
            if len < 100 {
                return Ok(());
            }
        }
    }

    pub async fn get_guild_from_remote(&self, guild_id: u64) -> Result<Guild, crate::Error> {
        self.api_client
            .send::<GetGuild>(&GetGuildRequest { guild_id })
            .await?
            .as_result()
            .map_err(crate::Error::context("get_guild_from_remote"))
    }

    pub async fn create_reaction(
        &self,
        reaction: &EmojiReactionDescriptor,
    ) -> Result<(), crate::Error> {
        self.api_client
            .send::<SendEmojiReaction>(reaction)
            .await?
            .as_result()
            .map_err(crate::Error::context("create_reaction"))?;
        Ok(())
    }

    pub async fn delete_reaction(
        &self,
        reaction: &EmojiReactionDescriptor,
    ) -> Result<(), crate::Error> {
        self.api_client
            .send::<DeleteEmojiReaction>(reaction)
            .await?
            .as_result()
            .map_err(crate::Error::context("delete_reaction"))?;
        Ok(())
    }

    pub async fn get_reaction_users(
        &self,
        reaction: &EmojiReactionDescriptor,
    ) -> Result<Vec<User>, crate::Error> {
        let mut req: GetEmojiReactionUserListRequest =
            GetEmojiReactionUserListRequest::new(reaction);
        let mut collector = vec![];
        loop {
            let resp = self
                .api_client
                .send::<GetEmojiReactionUserList>(&req)
                .await?
                .as_result()
                .map_err(crate::Error::context("get_reaction_users"))?;

            collector.extend(resp.users);
            if let Some(cookie) = resp.cookie {
                req.next(cookie);
            }
            if resp.is_end {
                collector.dedup();
                break Ok(collector);
            }
        }
    }
}
