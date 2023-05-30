use tokio::{
    sync::{broadcast, Notify},
    task::JoinHandle,
};
use tungstenite::{protocol::frame::coding::CloseCode, WebSocket};

use futures_util::{Future, SinkExt, StreamExt};
use std::sync::{
    atomic::{AtomicU32, AtomicU8, Ordering},
    Arc,
};
use tokio_tungstenite::{connect_async, WebSocketStream};

use crate::{
    client::ConnectType,
    model::{MessageAudited, MessageBotRecieved, MessageReaction},
    websocket::{DownloadPayload, Event, Ready, Resume, UploadPayload},
};

use super::ConnectOption;

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
    async fn auto_reconnect(
        self,
        event_broadcast_sender: broadcast::Sender<SeqEvent>,
        shutdown_signal: Arc<Notify>,
    ) {
        let mut conn_option = self;
        'connect: loop {
            let mut retry_count = 0;
            let retry_tolerance = conn_option.retry_times;
            let mut interval = tokio::time::interval(conn_option.retry_interval);
            match conn_option.connect().await {
                Ok(conn) => {
                    let mut cli = conn.luanch_client(event_broadcast_sender.clone()).await;
                    while let Some(task) = cli.download_bus_task.take() {
                        tokio::select! {
                            will_reconn = task => {
                                // 服务端关闭连接
                                if let Ok(true) = will_reconn {
                                    conn_option = cli.abort();
                                    continue 'connect;
                                }
                            }
                            _ = shutdown_signal.notified() => {
                                // 收到关闭信号
                                cli.abort();
                                break 'connect;
                            }
                        }
                    }
                    break 'connect;
                }
                Err(e) => {
                    retry_count += 1;
                    log::error!(
                        "ws client reconnect error ({retry_count}/{retry_tolerance}): {:?}",
                        e
                    );
                    if retry_count > retry_tolerance {
                        log::error!("ws client reconnect failed");
                        break 'connect;
                    }
                    interval.tick().await;
                }
            }
        }
    }

    async fn connect(&self) -> Result<ConnectionTokio, ConnectError> {
        use ConnectError::*;
        let (mut ws, _) = connect_async(&self.wss_gateway).await.map_err(Ws)?;

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
        match &self.connect_type {
            ConnectType::New(identify) => {
                token = identify.token.clone();
                let message = WsMessage::from(UploadPayload::Identify(identify.clone())).0;
                log::debug!("Sending identify: {:?}", &message);
                ws.send(message).await.map_err(Ws)?;
            }
            ConnectType::Reconnect(resume) => {
                token = resume.token.clone();
                ws.send(WsMessage::from(UploadPayload::Resume(resume.clone())).0)
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
            gateway: self.wss_gateway.clone(),
            option: self.clone(),
        })
    }
    pub fn run(
        self,
        shutdown_signal: impl Future<Output = ()> + Send + 'static,
    ) -> (broadcast::Receiver<SeqEvent>, JoinHandle<()>) {
        let (event_broadcast_sender, event_broadcast_rx) = broadcast::channel(1024);
        let shutdown_notifier = Arc::new(Notify::new());
        let shutdown_notifiee = shutdown_notifier.clone();
        let reconnect_task =
            tokio::spawn(self.auto_reconnect(event_broadcast_sender, shutdown_notifiee));
        tokio::spawn(async move {
            shutdown_signal.await;
            shutdown_notifier.notify_one();
        });
        (event_broadcast_rx, reconnect_task)
    }

    pub fn run_with_ctrl_c(self) -> (broadcast::Receiver<SeqEvent>, JoinHandle<()>) {
        let shutdown_signal = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to get CTRL+C signal");
        };
        self.run(shutdown_signal)
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
    /// gateway
    pub gateway: String,
    pub option: ConnectOption,
}

impl ConnectionTokio {
    pub async fn luanch_client(
        self,
        event_broadcast_sender: broadcast::Sender<SeqEvent>,
    ) -> WsClient {
        let latest_seq_raw = Arc::new(AtomicU32::new(0));
        let (mut tx, mut rx) = self.ws.split();

        // 收消息总线
        let latest_seq = latest_seq_raw.clone();
        let hb_counter_dl = Arc::new(AtomicU8::new(0));
        let hb_counter_hb = hb_counter_dl.clone();
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
                                    4009 | 4900..=4913 => {
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
                                latest_seq.store(seq, Ordering::Relaxed);
                                // 存活确认
                                hb_counter_dl.store(0, Ordering::SeqCst);
                                // 分发事件
                                event_broadcast_sender
                                    .send((*event, seq))
                                    .unwrap_or_default();
                            }
                            DownloadPayload::Heartbeat => {
                                // 收到服务端心跳
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
                                hb_counter_dl.store(0, Ordering::SeqCst);
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
        let latest_seq = latest_seq_raw.clone();
        let hb_interval = self.heartbeat_interval;
        let heartbeat = async move {
            use tokio::time::*;
            sleep(Duration::from_millis(hb_interval)).await;
            tx.send(
                WsMessage::from(UploadPayload::Heartbeat(Some(
                    latest_seq.load(Ordering::Relaxed),
                )))
                .0,
            )
            .await
            .unwrap_or_default();
            hb_counter_hb.fetch_add(1, Ordering::SeqCst);
        };

        let download_bus_task = tokio::spawn(download_bus);
        let heartbeat_task = tokio::spawn(heartbeat);

        WsClient {
            download_bus_task: Some(download_bus_task),
            heartbeat_task,
            latest_seq: latest_seq_raw,
            // hb_ack_missed: hb_ack_missed_raw,
            session_id: self.ready.session_id,
            option: self.option,
        }
    }
}

#[derive(Debug)]
pub struct WsClient {
    download_bus_task: Option<JoinHandle<bool>>,
    heartbeat_task: JoinHandle<()>,
    latest_seq: Arc<AtomicU32>,
    // hb_ack_missed: Arc<AtomicU8>,
    option: ConnectOption,
    pub session_id: String,
}

impl WsClient {
    #[inline]
    /// 最近序列号
    fn latest_seq(&self) -> u32 {
        self.latest_seq.load(Ordering::Relaxed)
    }

    /// 宕机，返回`ConnectOption`，可以用来进行重连
    fn abort(self) -> ConnectOption {
        self.heartbeat_task.abort();
        ConnectOption {
            connect_type: ConnectType::Reconnect(Resume {
                seq: self.latest_seq(),
                token: self.option.connect_type.get_token().to_owned(),
                session_id: self.session_id,
            }),
            ..self.option
        }
    }
}
