use serde_with::Seq;
use tokio::task::JoinHandle;
use tungstenite::{WebSocket, protocol::frame::coding::CloseCode};

use futures_util::{SinkExt, StreamExt};
use std::sync::{
    atomic::{AtomicU32, AtomicU8, Ordering},
    Arc,
};
use tokio_tungstenite::{connect_async, WebSocketStream};

use crate::{websocket::{DownloadPayload, Event, Identify, Ready, Resume, UploadPayload}, model::{MessageBotRecieved, MessageAudited, MessageReaction}, client::ConnectType};

pub type SeqEvent = (Event, u32);

#[derive(Debug, Clone)]
pub enum ClientEvent {
    AtMessageCreate(Arc<MessageBotRecieved>),
    MessageAuditPass(Arc<MessageAudited>),
    MessageAuditReject(Arc<MessageAudited>),
    MessgaeReactionAdd(Arc<MessageReaction>),
    MessgaeReactionRemove(Arc<MessageReaction>),
}

#[derive(Debug, Clone)]
pub struct SeqClientEvent {
    pub seq: u32,
    pub event: ClientEvent,
}

impl Message for SeqClientEvent {
    type Result = ();
}

#[repr(transparent)]
struct WsMessage(tungstenite::Message);

impl From<WsMessage> for Option<DownloadPayload> {
    fn from(val: WsMessage) -> Self {
        match val.0 {
            tungstenite::Message::Text(json_string) => {
                serde_json::from_str::<DownloadPayload>(&json_string).ok()
            }
            _ => None,
        }
    }
}

impl From<&UploadPayload> for WsMessage {
    fn from(upload: &UploadPayload) -> WsMessage {
        WsMessage(tungstenite::Message::Text(
            serde_json::to_string(upload).unwrap(),
        ))
    }
}

impl From<UploadPayload> for WsMessage {
    fn from(upload: UploadPayload) -> WsMessage {
        WsMessage::from(&upload)
    }
}



pub struct Connection {
    /// websocket 连接
    pub ws: WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
    /// 鉴权成功时服务端发回的`Ready`数据
    pub ready: Ready,
    /// 心跳间隔，单位：毫秒
    pub heartbeat_interval: u64,
}

#[derive(Debug)]
pub enum ConnectError {
    /// 第一条消息不是hello
    MissingHello,

    /// 鉴权失败
    AuthFailed,

    /// tungstenite 错误
    Ws(tungstenite::Error),
}

impl ConnectOption {
    pub async fn connect_tokio(self) -> Result<ConnectionTokio, ConnectError> {
        use ConnectError::*;
        let (mut ws, _) = connect_async(self.wss_gateway).await.map_err(Ws)?;

        // 1. 连接到 Gateway
        log::info!("Connected to gateway");
        let hello: Option<DownloadPayload> =
            WsMessage(ws.next().await.unwrap().map_err(Ws)?).into();

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
                let message = WsMessage::from(UploadPayload::Identify(identify)).0;
                log::debug!("Sending identify: {:?}", &message);
                ws.send(message).await.map_err(Ws)?;
            }
            ConnectType::Reconnect(resume) => {
                token = resume.token.clone();
                ws.send(WsMessage::from(UploadPayload::Resume(resume)).0)
                    .await
                    .map_err(Ws)?;
            }
        }

        // 3. 发送心跳
        log::info!("Sending heartbeat");
        let resp: Option<DownloadPayload> = WsMessage(ws.next().await.unwrap().map_err(Ws)?).into();

        let ready = *match resp {
            Some(DownloadPayload::Dispatch { event, seq: _ }) => {
                log::info!("ws client init recieve event: {:?}", event);
                match *event {
                    Event::Ready(ready) => {
                        ws.send(WsMessage::from(UploadPayload::Heartbeat(None)).0)
                            .await
                            .map_err(Ws)?;
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

        Ok(ConnectionTokio {
            ws,
            ready,
            heartbeat_interval,
            token,
        })
    }
}

pub struct ConnectionTokio {
    /// websocket 连接
    pub ws: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    /// 鉴权成功时服务端发回的`Ready`数据
    pub ready: Ready,
    /// 心跳间隔，单位：毫秒
    pub heartbeat_interval: u64,
    /// token
    pub token: String,
}

impl ConnectionTokio {
    pub async fn luanch_client(self) -> WsClient {
        let latest_seq = Arc::new(AtomicU32::new(0));
        let (mut tx, mut rx) = self.ws.split();

        // 上行消息总线，mpsc
        let (upload_bus_tx, mut upload_bus_rx) =
            tokio::sync::mpsc::unbounded_channel::<UploadPayload>();

        // 事件广播，broadcast
        let (event_broadcast_sender, _event_broadcast_reciever) =
            tokio::sync::broadcast::channel::<SeqEvent>(64);

        let event = event_broadcast_sender.clone();

        // 心跳应答缺失量：距离上次收到应答后发送的心跳量，broadcast
        let hb_ack_missed = Arc::new(AtomicU8::new(0));

        // 发消息总线
        let upload_bus = async move {
            while let Some(upload) = upload_bus_rx.recv().await {
                tx.send(WsMessage::from(upload).0).await.unwrap_or_default()
            }
        };

        // 收消息总线
        let latest_seq_clone = latest_seq.clone();
        let hb_ack_missed_clone = hb_ack_missed.clone();
        let download_bus = async move {
            match rx.next().await {
                None => {
                    // 服务端关闭连接
                    return false;
                }
                Some(Err(_e)) => {
                    // 服务端关闭连接
                    return false;
                }
                Some(Ok(message)) => {
                    if let tokio_tungstenite::tungstenite::Message::Close(cf) = message {
                        if let Some(cf) = cf {
                            log::debug!("ws client recieve close frame: {:?}", cf);
                            if let CloseCode::Library(code) = cf.code {
                                match code {
                                    4009 | 4900..=4913=> {
                                        log::debug!("ws will retry connect with code: {code}");
                                        return true;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        // 服务端关闭连接
                        return false;
                    }
                    let msg_bdg = message.clone();
                    if let Option::<DownloadPayload>::Some(download) = WsMessage(message).into() {
                        match download {
                            DownloadPayload::Dispatch { event, seq } => {
                                latest_seq_clone.store(seq, Ordering::Relaxed);
                                // 分发事件
                                event_broadcast_sender
                                    .send((*event, seq))
                                    .unwrap_or_default();
                            }
                            DownloadPayload::Heartbeat => {
                                // 收到服务端心跳，把应答缺失置零
                                hb_ack_missed_clone.store(0, Ordering::Release)
                            }
                            DownloadPayload::Reconnect => {
                                // 建立连接后应该不能收到重连通知
                                // 重连通知
                            }
                            DownloadPayload::InvalidSession => {
                                // 无效对话
                            }
                            DownloadPayload::Hello {
                                heartbeat_interval: _,
                            } => {
                                // 建立连接后应该不能收到hello消息
                            }
                            DownloadPayload::HeartbeatAck => {
                                // 收到服务端心跳，把应答缺失置零
                                hb_ack_missed_clone.store(0, Ordering::Release)
                            }
                        }
                    } else {
                        println!("无法解析的下行消息 {msg_bdg:?}")
                    }
                }
            }
            false
        };

        // spawn 心跳task
        let latest_seq_clone = latest_seq.clone();
        let hb_ack_missed_clone = hb_ack_missed.clone();
        let hb_interval = self.heartbeat_interval;
        let heartbeat = async move {
            use tokio::time::*;
            sleep(Duration::from_millis(hb_interval)).await;
            upload_bus_tx
                .send(UploadPayload::Heartbeat(Some(
                    latest_seq_clone.load(Ordering::Relaxed),
                )))
                .unwrap_or_default();
            // 应答缺失+1
            hb_ack_missed_clone.fetch_add(1, Ordering::Release);
        };

        let upload_bus_task = tokio::spawn(upload_bus);
        let download_bus_task = tokio::spawn(download_bus);
        let heartbeat_task = tokio::spawn(heartbeat);

        WsClient {
            upload_bus_task,
            download_bus_task,
            heartbeat_task,
            event,
            latest_seq,
            hb_ack_missed,
            shard: self.ready.shard,
            session_id: self.ready.session_id,
            token: self.token,
            subscribers: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct WsClient {
    upload_bus_task: JoinHandle<()>,
    download_bus_task: JoinHandle<bool>,
    heartbeat_task: JoinHandle<()>,
    event: tokio::sync::broadcast::Sender<SeqEvent>,
    latest_seq: Arc<AtomicU32>,
    hb_ack_missed: Arc<AtomicU8>,
    pub subscribers: Vec<Recipient<SeqClientEvent>>,
    pub shard: Option<[u32; 2]>,
    pub token: String,
    pub session_id: String,
}

impl WsClient {
    #[inline]
    /// 上次收到ack后，发送的心跳数，
    ///
    /// `u8`类型，发了255个心跳都没有应答，还是崩溃算了
    pub fn heartbeat_ack_missed(&self) -> u8 {
        self.hb_ack_missed.load(Ordering::Relaxed)
    }

    #[inline]
    /// 最近序列号
    pub fn latest_seq(&self) -> u32 {
        self.latest_seq.load(Ordering::Relaxed)
    }

    #[inline]
    /// 获取事件订阅
    pub fn subscribe_event(&self) -> tokio::sync::broadcast::Receiver<SeqEvent> {
        self.event.subscribe()
    }

    /// 宕机，返回`Resume`，可以用来进行重连
    pub fn abort(self) -> Resume {
        self.upload_bus_task.abort();
        self.download_bus_task.abort();
        self.heartbeat_task.abort();
        Resume {
            seq: self.latest_seq(),
            token: self.token,
            session_id: self.session_id,
        }
    }
}

use actix::prelude::*;

use super::ConnectOption;
impl Actor for WsClient {
    type Context = Context<Self>;
}

impl Handler<DownloadPayload> for WsClient {
    type Result = ();
    fn handle(&mut self, msg: DownloadPayload, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            DownloadPayload::Dispatch { event, seq } => {
                self.latest_seq.store(seq, Ordering::Relaxed);
                let e = match *event {
                    Event::AtMessageCreate(e) => {
                        SeqClientEvent {
                            seq,
                            event: ClientEvent::AtMessageCreate(e.into()),
                        }
                    },
                    Event::MessageAuditPass(e) => {
                        SeqClientEvent {
                            seq,
                            event: ClientEvent::MessageAuditPass(e.into()),
                        }
                    },
                    Event::MessageAuditReject(e) => {
                        SeqClientEvent {
                            seq,
                            event: ClientEvent::MessageAuditReject(e.into()),
                        }
                    },
                    Event::MessgaeReactionAdd(e) => {
                        SeqClientEvent {
                            seq,
                            event: ClientEvent::MessgaeReactionAdd(e.into()),
                        }
                    },
                    Event::MessgaeReactionRemove(e) => {
                        SeqClientEvent {
                            seq,
                            event: ClientEvent::MessgaeReactionRemove(e.into()),
                        }
                    },
                    Event::Ready(e) => {
                        log::debug!("ws client recieve ready event: {e:?}");
                        return;
                    },
                    Event::Resumed(e) => {
                        log::debug!("ws client recieve resumed event: {e}");
                        return;
                    },
                    Event::Unknown => {
                        log::debug!("ws client recieve unknown event: {:?}", event);
                        return;
                    },
                };
                // 分发事件
                for subscr in &self.subscribers {
                    subscr.do_send(e.clone());
                }
            }
            DownloadPayload::Heartbeat => {
                // 收到服务端心跳，把应答缺失置零
                self.hb_ack_missed.store(0, Ordering::Release)
            }
            DownloadPayload::Reconnect => {
                // 建立连接后应该不能收到重连通知
                // 重连通知
            }
            DownloadPayload::InvalidSession => {
                // 无效对话
            }
            DownloadPayload::Hello {
                heartbeat_interval: _,
            } => {
                // 建立连接后应该不能收到hello消息
            }
            DownloadPayload::HeartbeatAck => {
                // 收到服务端心跳，把应答缺失置零
                self.hb_ack_missed.store(0, Ordering::Release)
            }
        }
    }
}

