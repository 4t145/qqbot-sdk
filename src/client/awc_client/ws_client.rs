use awc::{ws::{Frame, Codec}, Ws};

use crate::{
    client::{ConnectOption, ConnectType},
    websocket::{DownloadPayload, UploadPayload, Event, Ready},
};
#[repr(transparent)]
struct WsMessage(awc::ws::Frame);
impl From<WsMessage> for Option<DownloadPayload> {
    fn from(val: WsMessage) -> Self {
        match val.0 {
            awc::ws::Frame::Text(json_string) => {
                serde_json::from_slice::<DownloadPayload>(&json_string).ok()
            }
            _ => None,
        }
    }
}

impl From<&UploadPayload> for awc::ws::Message {
    fn from(upload: &UploadPayload) -> awc::ws::Message {
        awc::ws::Message::Text(serde_json::to_string(upload).unwrap().into())
    }
}

pub enum ConnectError {
    /// 第一条消息不是hello
    MissingHello,

    /// 鉴权失败
    AuthFailed,

    WsProtocol(awc::error::WsProtocolError),
    WsClient(awc::error::WsClientError),
}
pub struct ConnectionAwc {
    /// websocket 连接
    pub ws: Framed<Box<dyn ConnectionIo>, Codec>,
    /// 鉴权成功时服务端发回的`Ready`数据
    pub ready: Ready,
    /// 心跳间隔，单位：毫秒
    pub heartbeat_interval: u64,
    /// token
    pub token: String,
}
impl ConnectOption {
    pub async fn connect_awc(self) -> Result<ConnectionAwc, ConnectError> {
        use futures_util::{SinkExt as _, StreamExt as _};

        let (_resp, mut ws) = awc::Client::new()
            .ws(self.wss_gateway)
            .connect()
            .await
            .map_err(ConnectError::WsClient)?;

        // 1. 连接到 Gateway
        log::info!("Connected to gateway");
        let hello: Option<DownloadPayload> =
            WsMessage(ws.next().await.unwrap().map_err(ConnectError::WsProtocol)?).into();

        let heartbeat_interval = match hello {
            Some(DownloadPayload::Hello { heartbeat_interval }) => heartbeat_interval,
            _ => return Err(ConnectError::MissingHello),
        };
        log::info!("Heartbeat interval: {:?}", heartbeat_interval);

        // 2. 鉴权连接
        log::info!("Identifying");
        let token;
        match self.connect_type {
            ConnectType::New(identify) => {
                token = identify.token.clone();
                let message = (&UploadPayload::Identify(identify)).into();
                log::debug!("Sending identify: {:?}", &message);
                ws.send(message).await.map_err(ConnectError::WsProtocol)?;
            }
            ConnectType::Reconnect(resume) => {
                token = resume.token.clone();
                let message = (&UploadPayload::Resume(resume)).into();

                ws.send(message).await.map_err(ConnectError::WsProtocol)?;
            }
        }

        // 3. 发送心跳
        log::info!("Sending heartbeat");
        let resp: Option<DownloadPayload> = WsMessage(ws.next().await.unwrap().map_err(ConnectError::WsProtocol)?).into();

        let ready = *match resp {
            Some(DownloadPayload::Dispatch { event, seq: _ }) => {
                log::info!("ws client init recieve event: {:?}", event);
                match *event {
                    Event::Ready(ready) => {
                        ws.send((&UploadPayload::Heartbeat(None)).into())
                            .await
                            .map_err(ConnectError::WsProtocol)?;
                        ready
                    }
                    _ => return Err(ConnectError::AuthFailed),
                }
            }
            e => {
                log::info!("fail to get response {e:?}");
                return Err(ConnectError::AuthFailed);
            }
        };
        Ok(ConnectionAwc {
            ws,
            ready,
            heartbeat_interval,
            token,
        })
    }
}

pub struct AwcWsClient {}
