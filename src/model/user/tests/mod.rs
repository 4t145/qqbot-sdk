use super::*;

#[test]
fn deserialize_user() {
    let json = include_str!("users.json");
    let val = serde_json::from_str::<Vec<User>>(json).unwrap();
    drop(val);
    // println!("{:#?}", val);
}