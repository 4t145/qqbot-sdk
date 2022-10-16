use qqbot_sdk::{
    client::{
        reqwest_client::*,
        tungstenite_client::*,
    },
    api::{
        Authorization,
        websocket::{
            Gateway,
        }
    }, 
    websocket::{
        Identify,
        Intends,
    },
};

use reqwest::Error;

#[tokio::main]
async fn main() {
    async_main().await.unwrap();
}

async fn async_main() -> Result<(), Error> {
    // 启动webapi client
    let auth = Authorization::Bot { app_id: "000011112222", token: "AAAABBBBCCCCDDDDEEEEFFFF" };
    let token = auth.token();
    let webapi_client = ApiClient::new(auth);
    let url = webapi_client
        .send::<Gateway>(&()).await?
        .as_result().unwrap().url;

    // 启动websocket client
    let identify = Identify {
        token: token,
        intents: Intends::PUBLIC_GUILD_MESSAGES | Intends::GUILD_MESSAGE_REACTIONS,
        shard: None,
        properties: std::collections::HashMap::new(),
    };

    // ws连接设置
    let connect_option = ConnectOption {
        wss_gateway: url,
        connect_type: ConnectType::New(identify),
    };

    // ws连接
    let ws_connect = connect_option.connect_tokio().await.unwrap();

    // ws启动客户端
    let ws_client = ws_connect.luanch_client().await;

    // 事件
    let mut evt_rx = ws_client.subscribe_event();

    while let Ok((evt, seq)) = evt_rx.recv().await {
        println!("收到事件[{seq:?}]{evt:?}");
    }

    return Ok(())
}