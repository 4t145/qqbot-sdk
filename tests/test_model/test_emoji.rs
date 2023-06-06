#[cfg(test)]
pub mod tests {
    use super::*;
    const TEST_CASE_PAIRS: &[(Emoji, &str)] = &[
        (Emoji::System(4), r#"{"type":1,"id":"4"}"#),
        (Emoji::Raw(127801), r#"{"type":2,"id":"127801"}"#),
    ];
    #[test]
    fn deserialize_test() {
        for (emoji, json) in TEST_CASE_PAIRS {
            let e: Emoji = serde_json::from_str(json).unwrap();
            assert_eq!(emoji, &e);
        }
    }

    #[test]
    fn serialize_test() {
        for (emoji, json) in TEST_CASE_PAIRS {
            let e = serde_json::to_string(emoji).unwrap();
            assert_eq!(json, &e);
        }
    }
}