use ed25519_dalek::{
    SECRET_KEY_LENGTH, Signature, SigningKey, Verifier, VerifyingKey, ed25519::signature::Signer,
};

pub fn gen_ed25519_keys(bot_secret: &str) -> (SigningKey, VerifyingKey) {
    let mut seed = [0u8; SECRET_KEY_LENGTH];
    for (index, bytes) in bot_secret
        .as_bytes()
        .iter()
        .cycle()
        .take(SECRET_KEY_LENGTH)
        .enumerate()
    {
        seed[index] = *bytes;
    }
    let signing_key = SigningKey::from_bytes(&seed);
    let vk = signing_key.verifying_key();
    (signing_key, vk)
}

pub fn verify_signature(bot_secret: &str, message: &[u8], signature: &Signature) -> bool {
    let (_, vk) = gen_ed25519_keys(bot_secret);
    vk.verify(message, signature).is_ok()
}

pub fn sign(bot_secret: &str, message: &[u8]) -> Signature {
    let (sk, _) = gen_ed25519_keys(bot_secret);
    sk.sign(message)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SIGNATURE_LENGTH;

    use super::*;
    #[test]
    fn test_gen_pubkey() {
        // follow https://bot.q.qq.com/wiki/develop/api-v2/dev-prepare/interface-framework/sign.html
        const PRIVATE_KEY: [u8; 64] = [
            110, 97, 79, 67, 48, 111, 99, 81, 69, 51, 115, 104, 87, 76, 65, 102, 102, 102, 86, 76,
            66, 49, 114, 104, 89, 80, 71, 55, 110, 97, 79, 67, 215, 195, 98, 254, 120, 174, 248,
            31, 242, 50, 135, 180, 147, 98, 139, 93, 176, 42, 60, 79, 227, 11, 33, 94, 77, 25, 96,
            155, 93, 118, 103, 58,
        ];
        const PUBLIC_KEY: [u8; 32] = [
            215, 195, 98, 254, 120, 174, 248, 31, 242, 50, 135, 180, 147, 98, 139, 93, 176, 42, 60,
            79, 227, 11, 33, 94, 77, 25, 96, 155, 93, 118, 103, 58,
        ];
        let (sk, vk) = gen_ed25519_keys("naOC0ocQE3shWLAfffVLB1rhYPG7");
        assert_eq!(sk.to_keypair_bytes(), PRIVATE_KEY);
        assert_eq!(vk.as_bytes(), &PUBLIC_KEY);
    }

    #[test]
    fn test_verify() -> Result<(), Box<dyn std::error::Error>> {
        // follow https://github.com/tencent-connect/botgo/blob/fe31c0dfe469001e0f783d2f07e7de7bd08b403f/interaction/signature/interaction_test.go
        let bot_secret = "123456abcdef";
        let body = r#"{"id":"ROBOT1.0_veoihSEXDc8Q.g-6eLpNIa11bH8MisOjn-m-LKxCPntMk6exUXgcWCGpVO7L2QKTNZzjZzFFDSbiOFcqAPWyVA!!","content":"哦一下","timestamp":"2024-10-15T16:33:15+08:00","author":{"id":"675860273","user_openid":"675860273"}}"#;
        let timestamp = "1728981195";
        let message = format!("{}{}", timestamp, body);
        let signature = "e949b5b94ef4103df903fb031d1d16e358db3db83e79e117edd404c8508be3ce8a76d7bad1bed353194c126a1a5915b4ad8b5288c1191cc53a12acffccd82004";
        let signature_generated = hex::encode(
            gen_ed25519_keys(bot_secret)
                .0
                .sign(message.as_bytes())
                .to_bytes(),
        );
        println!("{}", signature_generated);
        let signature: [u8; SIGNATURE_LENGTH] = hex::decode(signature)?
            .try_into()
            .expect("signature length error");
        let signature = Signature::from_bytes(&signature);
        assert!(verify_signature(bot_secret, message.as_bytes(), &signature));
        Ok(())
    }
}
