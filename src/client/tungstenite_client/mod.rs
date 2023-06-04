use tokio::{
    sync::{broadcast, Notify},
    task::JoinHandle,
};
use tungstenite::{protocol::frame::coding::CloseCode, WebSocket};

use futures_util::{Future, SinkExt, Stream, StreamExt};
use std::{
    error::Error,
    fmt::Display,
    sync::{
        atomic::{AtomicU32, AtomicU8, Ordering},
        Arc,
    },
};
use tokio_tungstenite::{connect_async, WebSocketStream};

use crate::{
    client::ConnectType,
    model::{MessageAudited, MessageBotRecieved, MessageReaction},
    websocket::{DownloadPayload, Event, Ready, Resume, UploadPayload},
};

use super::{ClientEvent, ConnectConfig, Connection, ConnectionState};

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

pub struct Connected {
    pub(crate) session_id: String,
    pub(crate) recv_task: JoinHandle<(bool, bool)>,
    pub(crate) heartbeat_task: JoinHandle<()>,
}
pub enum TungsteniteConnectionState {
    Connecting,
    Connected(Connected),
    Reconnecting,
    Disconnected {
        resume: Option<Resume>,
        allow_identify: bool,
    },
    Guaranteed,
}

impl Into<ConnectionState> for &TungsteniteConnectionState {
    fn into(self) -> ConnectionState {
        match self {
            TungsteniteConnectionState::Connecting => ConnectionState::Connecting,
            TungsteniteConnectionState::Connected { .. } => ConnectionState::Connected,
            TungsteniteConnectionState::Reconnecting => ConnectionState::Reconnecting,
            TungsteniteConnectionState::Disconnected { .. } => ConnectionState::Disconnected,
            TungsteniteConnectionState::Guaranteed => ConnectionState::Guaranteed,
        }
    }
}

pub struct TungsteniteConnection {
    state: TungsteniteConnectionState,
    config: ConnectConfig,
    event_sender: broadcast::Sender<ClientEvent>,
    last_sequence: Arc<AtomicU32>,
}
type WsRx = futures_util::stream::SplitStream<
    WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;
type WsTx = futures_util::stream::SplitSink<
    WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Message,
>;
type Ws = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
impl TungsteniteConnection {
    fn start_loop(
        &self,
        heartbeat_interval: u64,
        ws: Ws,
    ) -> (JoinHandle<()>, JoinHandle<(bool, bool)>) {
        let (tx, rx) = ws.split();
        let heartbeat_task = self.hb_loop(heartbeat_interval, tx);
        let recv_task = self.recv_loop(rx);
        (heartbeat_task, recv_task)
    }
    fn hb_loop(&self, heartbeat_interval: u64, mut tx: WsTx) -> JoinHandle<()> {
        let last_seq = self.last_sequence.clone();
        let heartbeat_interval = tokio::time::Duration::from_millis(heartbeat_interval);
        // spawn 心跳task
        tokio::spawn(async move {
            use tokio::time::*;
            let mut interval = interval(heartbeat_interval);
            loop {
                interval.tick().await;
                let latest_seq = last_seq.load(Ordering::SeqCst);
                let message = WsMessage::from(UploadPayload::Heartbeat(Some(latest_seq))).0;
                if let Err(e) = tx.send(message).await {
                    log::error!("send heartbeat failed: {e}");
                }
            }
        })
    }
    fn recv_loop(&self, mut rx: WsRx) -> JoinHandle<(bool, bool)> {
        let last_seq = self.last_sequence.clone();
        let event_tx = self.event_sender.clone();
        let recv_jh: JoinHandle<(bool, bool)> = tokio::spawn(async move {
            while let Some(maybe_message) = rx.next().await {
                match maybe_message {
                    Err(e) => {
                        // 服务端关闭连接
                        log::error!("ws client recieve error: {:?}", e);
                        break;
                    }
                    Ok(message) => {
                        if let tokio_tungstenite::tungstenite::Message::Close(cf) = message {
                            if let Some(cf) = cf {
                                log::debug!("ws client recieve close frame: {:?}", cf);
                                if let CloseCode::Library(code) = cf.code {
                                    macro_rules! match_close_code {
                                        ($val:expr => {
                                            $($code:pat => ($can_resume:expr, $can_identify:expr, $message:literal),)*
                                        }) => {
                                            match $val {
                                                $($code => {
                                                    log::error!("ws error, {code}:{message}", code=stringify!($code), message=$message);
                                                    return ($can_resume, $can_identify)
                                                }),*
                                                _ => {
                                                    log::error!("ws error, {code}:{message}", code=stringify!($val), message="未知错误");
                                                    return (false, false)
                                                }
                                            }
                                        };
                                    }
                                    match_close_code! {
                                        code => {
                                            4001=>(false, false,"无效的 opcode"),
                                            4002=>(false, false,"无效的 payload"),
                                            4006=>(false, true,"无效的 session"),
                                            4007=>(false, true,  "seq 错误"),
                                            4008=>(true,  true,"发送 payload 过快，请重新连接，并遵守连接后返回的频控信息"),
                                            4009=>(true,  true,"连接过期，请重连并执行 resume 进行重新连接"),
                                            4010=>(false, false,"无效的 shard"),
                                            4011=>(false, false,"连接需要处理的 guild 过多，请进行合理的分片"),
                                            4012=>(false, false,"无效的 version")  ,
                                            4013=>(false, false,"无效的 intent")  ,
                                            4014=>(false, false,"无效的加密模式")  ,
                                            4900..=4913=>(false, true,"内部错误，请重连") ,
                                            4914=>(false, false,"机器人已下架,只允许连接沙箱环境,请断开连接,检验当前连接环境"),
                                            4915=>(false, false,"机器人已封禁,不允许连接,请断开连接,申请解封后再连接"),
                                        }
                                    }
                                }
                            }
                            // 服务端关闭连接
                            return (false, true);
                        }
                        let msg_bdg = message.clone();
                        if let Option::<DownloadPayload>::Some(download) = WsMessage(message).into()
                        {
                            match download {
                                DownloadPayload::Dispatch { event, seq } => {
                                    log::debug!("ws client recieve event: {:?}", event);
                                    last_seq.store(seq, Ordering::Relaxed);
                                    let Ok(client_event): Result<ClientEvent, ()> = (*event).try_into() else {
                                        continue;
                                    };
                                    // 分发事件
                                    event_tx
                                        .send(client_event)
                                        .map_err(|e| {
                                            log::error!("ws client send event failed: {e}")
                                        })
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
                                    // 收到服务端心跳
                                }
                            }
                        } else {
                            log::warn!("无法解析的下行消息 {msg_bdg:?}")
                        }
                    }
                }
            }
            log::debug!("ws client download bus exit");
            (false, false)
        });
        recv_jh
    }
    async fn prepare_ws(&self) -> Result<(Ws, u64), TungsteniteConnectionError> {
        let (mut ws, _) = connect_async(&self.config.wss_gateway).await?;
        // 1. 连接到 Gateway
        log::info!("Connected to gateway");
        let Some(first_ws_message) = ws.next().await else {
            // 这其实不太可能发生
            // 如果发生了，可能ws连接已经被关闭
            return Err(TungsteniteConnectionError::WsStreamClosed);
        };

        let Some(DownloadPayload::Hello { heartbeat_interval }): Option<DownloadPayload> = WsMessage(first_ws_message?).into() else {
            return Err(TungsteniteConnectionError::MissingHello);
        };

        log::info!("Heartbeat interval: {:?}", heartbeat_interval);
        Ok((ws, heartbeat_interval))
    }
}
#[async_trait::async_trait]
impl Connection for TungsteniteConnection {
    type Error = TungsteniteConnectionError;
    fn new(config: ConnectConfig, event_sender: broadcast::Sender<ClientEvent>) -> Self {
        Self {
            state: TungsteniteConnectionState::Disconnected {
                resume: None,
                allow_identify: true,
            },
            config,
            event_sender,
            last_sequence: Arc::new(AtomicU32::new(0)),
        }
    }

    fn get_state(&self) -> ConnectionState {
        (&self.state).into()
    }

    fn get_config(&self) -> &ConnectConfig {
        &self.config
    }

    fn confict_state_err(state: ConnectionState, expected: ConnectionState) -> Self::Error {
        TungsteniteConnectionError::StateConfict {
            current: state,
            expected,
        }
    }

    async fn connect_inner(&mut self) -> Result<(), Self::Error> {
        self.state = TungsteniteConnectionState::Connecting;
        // 1. 连接到 Gateway
        let (mut ws, heartbeat_interval) = self.prepare_ws().await?;

        // 2. 鉴权连接
        log::info!("Identifying");
        let identify = self.config.identify.clone();
        let message = WsMessage::from(UploadPayload::Identify(identify)).0;
        log::debug!("Sending identify: {:?}", &message);
        ws.send(message).await?;

        // 3. 发送心跳
        log::info!("Sending heartbeat");
        let resp_message = ws
            .next()
            .await
            .ok_or(TungsteniteConnectionError::WsStreamClosed)??;
        log::debug!("get response message: {resp_message}");

        let Some(resp_pld): Option<DownloadPayload> = WsMessage(resp_message).into() else {
            log::error!("get invalid response message");
            return Err(TungsteniteConnectionError::AuthFailed);
        };

        let DownloadPayload::Dispatch { event, seq } = resp_pld else {
            log::error!("get unexpected response payload: {resp_pld:?}");
            return Err(TungsteniteConnectionError::AuthFailed);
        };

        let Event::Ready(ready) = *event else {
            log::error!("get unexpected event: {event:?}");
            return Err(TungsteniteConnectionError::AuthFailed);
        };

        let last_seq_raw = self.last_sequence.clone();
        last_seq_raw.store(seq, Ordering::SeqCst);

        let (heartbeat_jh, recv_jh) = self.start_loop(heartbeat_interval, ws);

        self.state = TungsteniteConnectionState::Connected(Connected {
            session_id: ready.session_id.to_string(),
            recv_task: recv_jh,
            heartbeat_task: heartbeat_jh,
        });
        Ok(())
    }

    async fn reconnect_inner(&mut self) -> Result<(), Self::Error> {
        let mut state = TungsteniteConnectionState::Reconnecting;
        std::mem::swap(&mut state, &mut self.state);
        let TungsteniteConnectionState::Disconnected { resume, allow_identify } = state else {
            return Err(Self::confict_state_err((&state).into(), ConnectionState::Disconnected));
        };
        let (mut ws, heartbeat_interval) = self.prepare_ws().await?;
        if let Some(resume) = resume {
            log::info!("Resuming");
            let message = WsMessage::from(UploadPayload::Resume(resume.clone())).0;
            log::debug!("Sending identify: {:?}", &message);
            ws.send(message).await?;
            let (heartbeat_task, recv_task) = self.start_loop(heartbeat_interval, ws);
            self.state = TungsteniteConnectionState::Connected(Connected {
                session_id: resume.session_id.to_string(),
                recv_task,
                heartbeat_task,
            });
            Ok(())
        } else if allow_identify {
            self.connect_inner().await
        } else {
            log::error!("can not reconnect");
            Err(TungsteniteConnectionError::CantReconnect)
        }
    }

    async fn wait_disconect_inner(&mut self) -> Result<(), Self::Error> {
        let mut retry_interval = tokio::time::interval(self.config.retry_interval);
        let mut retry_times = 0;
        let retry_max_times = self.config.retry_times;
        loop {
            let mut state = TungsteniteConnectionState::Guaranteed;
            std::mem::swap(&mut state, &mut self.state);
            let TungsteniteConnectionState::Connected(Connected { session_id, recv_task, heartbeat_task }) = state else {
                return Err(Self::confict_state_err((&state).into(), ConnectionState::Disconnected));
            };
            match recv_task.await {
                Ok((resume, indentify)) => {
                    self.state = TungsteniteConnectionState::Disconnected {
                        resume: resume.then_some(Resume {
                            token: self.config.identify.token.clone(),
                            session_id,
                            seq: self.last_sequence.load(Ordering::SeqCst),
                        }),
                        allow_identify: indentify,
                    };
                    'retry: loop {
                        match self.reconnect_inner().await {
                            Ok(()) => {
                                log::info!("connect success");
                                break 'retry;
                            }
                            Err(e) => {
                                retry_times += 1;
                                log::error!("connect failed({retry_times}/{retry_max_times}): {:?}", e);
                                if retry_times >= retry_max_times {
                                    log::error!("reconnect failed: retry times exceed");
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("recv_task join error: {:?}", e);
                    return Err(Self::Error::Ws(tungstenite::Error::ConnectionClosed));
                }
            }
            retry_interval.tick().await;
        }

    }
}

// pub struct Connection {
//     /// websocket 连接
//     pub ws: WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
//     /// 鉴权成功时服务端发回的`Ready`数据
//     pub ready: Ready,
//     /// 心跳间隔，单位：毫秒
//     pub heartbeat_interval: u64,
// }

#[derive(Debug)]
pub enum TungsteniteConnectionError {
    CantReconnect,
    /// websocket连接状态错误
    StateConfict {
        current: ConnectionState,
        expected: ConnectionState,
    },
    /// websocket连接已关闭
    WsStreamClosed,

    /// 第一条消息不是hello
    MissingHello,

    /// 鉴权失败
    AuthFailed,

    /// tungstenite 错误
    Ws(tungstenite::Error),
}

impl From<tungstenite::Error> for TungsteniteConnectionError {
    fn from(err: tungstenite::Error) -> Self {
        Self::Ws(err)
    }
}

impl Display for TungsteniteConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TungsteniteConnectionError::StateConfict { current, expected } => write!(
                f,
                "Conflict state: expect: {}, currunt: {}",
                expected, current
            ),
            TungsteniteConnectionError::WsStreamClosed => {
                write!(f, "Websocket stream already closed")
            }
            TungsteniteConnectionError::MissingHello => write!(f, "Missing hello"),
            TungsteniteConnectionError::AuthFailed => write!(f, "Auth failed"),
            TungsteniteConnectionError::Ws(err) => write!(f, "Tungstenite error: {}", err),
            TungsteniteConnectionError::CantReconnect => write!(f, "Can't reconnect"),
        }
    }
}
impl Error for TungsteniteConnectionError {}