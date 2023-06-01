/// 默认域名：正式环境
pub static PROD_DOMAIN: &str = "https://api.sgroup.qq.com";
/// 默认域名：沙箱环境
pub static SANDBOX_DOMAIN: &str = "https://sandbox.api.sgroup.qq.com";

pub fn domain() -> &'static str {
    let sandbox_mode = std::env::var("MODE").unwrap_or("SANDBOX".to_string());
    match sandbox_mode.as_str() {
        "SANDBOX" => SANDBOX_DOMAIN,
        "PROD" => PROD_DOMAIN,
        _ => SANDBOX_DOMAIN,
    }
}
