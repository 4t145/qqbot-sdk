use crate::{
    api::{
        guild::{GetGuild, GetGuildRequest},
        message::{PostMessage, PostMessageRequest},
        reaction::{
            DeleteEmojiReaction, EmojiReactionDescriptor, GetEmojiReactionUserList,
            GetEmojiReactionUserListRequest, SendEmojiReaction,
        },
        user::GetMe,
    },
    client::audit_hook::AuditResult,
    model::{Guild, MessageSend, User},
};

use super::*;

impl Bot {
    pub fn cache(&self) -> BotCache {
        self.cache.clone()
    }
    pub async fn send_message_public(
        &self,
        channel_id: u64,
        message: &MessageSend<'_>,
    ) -> Result<crate::model::MessageAudited, BotError> {
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
                            return Err(BotError::BadRequest(f));
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
                            .audit_hook_pool
                            .create(audit_hook_id)
                            .await
                            .await_result()
                            .await;
                        match res {
                            AuditResult::Pass(a) => Ok(a),
                            AuditResult::Reject(a) => Ok(a),
                            AuditResult::Timeout => Err(BotError::AuditTimeout),
                        }
                    }
                    _ => Err(BotError::BadRequest(f)),
                }
            }
        }
    }
    pub async fn send_message(
        &self,
        channel_id: u64,
        message: &MessageSend<'_>,
    ) -> Result<crate::model::MessageBotRecieved, BotError> {
        let request = PostMessageRequest::new(channel_id, message);
        let resp = self
            .api_client
            .send::<PostMessage>(&request)
            .await?
            .as_result()?;
        Ok(resp)
    }

    pub async fn about_me(&self) -> Result<crate::model::User, BotError> {
        self.api_client
            .send::<GetMe>(&())
            .await
            .map_err(BotError::ApiError)?
            .as_result()
            .map_err(BotError::BadRequest)
    }

    pub async fn fetch_my_guilds(&self) -> Result<(), BotError> {
        let mut req = crate::api::user::GetMyGuildsRequest::default();
        loop {
            let guilds = self
                .api_client
                .send::<crate::api::user::GetMyGuilds>(&req)
                .await
                .map_err(BotError::ApiError)?
                .as_result()
                .map_err(BotError::BadRequest)?;
            req.after = guilds.last().map(|x| x.id);
            let len = guilds.len();
            log::debug!("guilds: {:#?}", guilds);
            self.cache.cache_many_guilds(guilds).await;
            if len < 100 {
                return Ok(());
            }
        }
    }

    pub async fn get_guild_from_remote(&self, guild_id: u64) -> Result<Guild, BotError> {
        self.api_client
            .send::<GetGuild>(&GetGuildRequest { guild_id })
            .await
            .map_err(BotError::ApiError)?
            .as_result()
            .map_err(BotError::BadRequest)
    }

    pub async fn create_reaction(
        &self,
        reaction: &EmojiReactionDescriptor,
    ) -> Result<(), BotError> {
        self.api_client
            .send::<SendEmojiReaction>(reaction)
            .await
            .map_err(BotError::ApiError)?
            .as_result()
            .map_err(BotError::BadRequest)?;
        Ok(())
    }

    pub async fn delete_reaction(
        &self,
        reaction: &EmojiReactionDescriptor,
    ) -> Result<(), BotError> {
        self.api_client
            .send::<DeleteEmojiReaction>(reaction)
            .await
            .map_err(BotError::ApiError)?
            .as_result()
            .map_err(BotError::BadRequest)?;
        Ok(())
    }

    pub async fn get_reaction_users(
        &self,
        reaction: &EmojiReactionDescriptor,
    ) -> Result<Vec<User>, BotError> {
        let mut req: GetEmojiReactionUserListRequest =
            GetEmojiReactionUserListRequest::new(reaction);
        let mut collector = vec![];
        loop {
            let resp = self
                .api_client
                .send::<GetEmojiReactionUserList>(&req)
                .await
                .map_err(BotError::ApiError)?
                .as_result()
                .map_err(BotError::BadRequest)?;

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
