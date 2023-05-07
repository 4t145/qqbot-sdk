use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum MessageMarkdown {
    Template {
        template_id: i32,
        custom_template_id: String,
        params: MessageMarkdownParams,
    },
    Raw {
        content: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageMarkdownParams {
    key: String,
    values: [String; 1],
}
