use serde::Deserialize;
use serde_json::json;

use crate::websocket::{Event, Opcode, Payload};

#[derive(Deserialize, Clone, Debug)]
#[serde(from = "Payload")]
pub enum DownloadPayload {
    Dispatch { event: Box<Event>, seq: u32 },
    Heartbeat,
    Reconnect,
    InvalidSession,
    Hello { heartbeat_interval: u64 },
    HeartbeatAck,
}

impl From<Payload> for DownloadPayload {
    fn from(payload: Payload) -> Self {
        match payload.opcode {
            Opcode::Dispatch => {
                log::debug!("recieve dispatch payload: {payload:?}");
                let json_value = json!({
                    "tag": payload.tag,
                    "data": payload.data,
                });
                log::trace!("convert payload to download payload json: {json_value}");
                let event = match serde_json::from_value::<Event>(json_value) {
                    Ok(download_payload) => download_payload,
                    Err(e) => {
                        log::warn!(
                            "failed to convert payload to download payload json, error: {e}"
                        );
                        Event::Unknown
                    }
                };
                DownloadPayload::Dispatch {
                    event: Box::new(event),
                    seq: payload.seq.unwrap_or_default(),
                }
            }
            Opcode::Heartbeat => DownloadPayload::Heartbeat,
            Opcode::Reconnect => DownloadPayload::Reconnect,
            Opcode::InvalidSession => DownloadPayload::InvalidSession,
            Opcode::Hello => DownloadPayload::Hello {
                heartbeat_interval: payload
                    .data
                    .expect("hello payload should have data")
                    .get("heartbeat_interval")
                    .expect("hello payload should have heartbeat_interval")
                    .as_u64()
                    .expect("hello payload heartbeat_interval should be able to convert into u64"),
            },
            Opcode::HeartbeatAck => DownloadPayload::HeartbeatAck,
            code => panic!("recieve websocket payload of unsupport opcode {code:?}"),
        }
    }
}