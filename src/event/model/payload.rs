use serde::{Deserialize, Serialize};

use super::{Event, Opcode};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeneralPayload {
    pub(crate) id: String,
    #[serde(rename = "op")]
    pub(crate) opcode: Opcode,
    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    pub(crate) seq: Option<u32>,
    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub(crate) event_type: Option<String>,
    #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpCallbackValidationRequest {
    plain_token: String,
    event_ts: String,
}

impl HttpCallbackValidationRequest {
    pub fn valid(self, bot_secret: &str) -> HttpCallbackValidationResponse {
        let message = format!("{}{}", self.event_ts, self.plain_token);
        let signature = crate::utils::sign(bot_secret, message.as_bytes());
        HttpCallbackValidationResponse {
            plain_token: self.plain_token,
            signature: signature.to_string(),
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpCallbackValidationResponse {
    plain_token: String,
    signature: String,
}

pub enum InboundPayloadKind {
    HttpCallbackValidation(HttpCallbackValidationRequest),
    Dispatch(Event),
}

impl GeneralPayload {
    pub fn into_inbound(self) -> crate::Result<InboundPayloadKind> {
        match self.opcode {
            Opcode::Dispatch => {
                let json_value = serde_json::json!({
                    "kind": self.event_type,
                    "data": self.data,
                });
                let event = serde_json::from_value::<Event>(json_value)
                    .map_err(crate::Error::context("converting payload to event"))?;
                Ok(InboundPayloadKind::Dispatch(event))
            }
            Opcode::HttpCallbackValidation => {
                let data = self.data.ok_or(crate::Error::unexpected(
                    "http callback validation data is missing",
                ))?;
                let request: HttpCallbackValidationRequest = serde_json::from_value(data).map_err(
                    crate::Error::context("converting payload to http callback validation"),
                )?;
                Ok(InboundPayloadKind::HttpCallbackValidation(request))
            }
            _ => {
                tracing::warn!("unexpected inbound opcode: {:?}", self.opcode);
                Err(crate::Error::unexpected(format!(
                    "unexpected inbound opcode {}",
                    self.opcode
                )))
            }
        }
    }

    pub fn new_http_callback_ack(&self) -> Self {
        Self {
            id: self.id.clone(),
            opcode: Opcode::HttpCallbackAck,
            seq: None,
            event_type: None,
            data: None,
        }
    }
}
