use qqbot_sdk::websocket::{Payload, DownloadPayload};

#[test]
fn deserialize_message_reaction_remove() {    
    std::env::set_var("RUST_LOG", "info,qqbot_sdk=trace");
    env_logger::builder().is_test(true).try_init().unwrap();
    let json_str = include_str!("message_reaction_remove.json");
    let pld = serde_json::from_str::<Vec<Payload>>(json_str).unwrap();
    let dld_pld = pld.into_iter().map(DownloadPayload::from).collect::<Vec<_>>();
    for pld in dld_pld {
        match pld {
            DownloadPayload::Dispatch { event, seq } => {
                match *event {
                    qqbot_sdk::websocket::Event::MessageReactionRemove(m) => {
                        log::info!("seq: {:?}, event: {:?}", seq, m);
                    },
                    _ => panic!("unexpect event: {:?}", event)
                }
            },
            _ => panic!("unexpect payload: {:?}", pld)
        }
    }
}