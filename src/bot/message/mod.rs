use crate::model::{MessageBotRecieved, MessageId, MessageReference, MessageSend};

#[derive(Debug, Default)]
pub struct MessageBuilder<'a> {
    content: Option<&'a str>,
    message_reference: Option<MessageReference>,
    reply_to: Option<MessageId>,
}

impl<'a> MessageBuilder<'a> {
    pub fn refer(mut self, message_id: u64) -> Self {
        if let Some(mut refer) = self.message_reference {
            refer.message_id = message_id;
            self.message_reference = Some(refer);
        } else {
            self.message_reference = Some(MessageReference::new(message_id));
        }
        self
    }
    pub fn content(mut self, content: impl Into<&'a str>) -> Self {
        self.content = Some(content.into());
        self
    }
    pub fn reply_to(mut self, message: &'a MessageBotRecieved) -> Self {
        self.reply_to = Some(message.id.clone());
        self
    }
    pub fn reply_to_id(mut self, message_id: MessageId) -> Self {
        self.reply_to = Some(message_id);
        self
    }
    pub fn build(self) -> Result<MessageSend<'a>, String> {
        let mut message = MessageSend::default();
        if let Some(content) = self.content {
            message.content = Some(content);
        } else {
            return Err("No content".to_string());
        }
        if let Some(message_reference) = self.message_reference {
            message.message_reference = Some(message_reference);
        }
        if let Some(reply_to) = self.reply_to {
            message.msg_id = Some(reply_to);
        }
        Ok(message)
    }
}
