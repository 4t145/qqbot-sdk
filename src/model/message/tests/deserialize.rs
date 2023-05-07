use super::*;

#[test]
fn message_recieved() {
    let json = include_str!("message_recieved.json");
    let val = serde_json::from_str::<Vec<MessageRecieved>>(json).unwrap();
    drop(val);
    // println!("{:#?}", val);
}

