use serde::{Deserialize, Serialize};
use serde_with::{serde_as, BoolFromInt};
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Role {
    /// 身份组ID
    pub id: String,
    /// 名称
    pub name: String,
    /// ARGB的HEX十六进制颜色值转换后的十进制数值
    pub color: u32,
    #[serde_as(as = "BoolFromInt")]
    /// 是否在成员列表中单独展示: 0-否, 1-是
    pub hoist: bool,
    /// 人数
    pub number: u32,
    /// 成员上限
    pub member_limit: u32,
}

/// 默认的身份组id
pub struct DefaultRoleId {}

macro_rules! def {
    ($($ident:ident, $value:expr, #[$doc:meta])*) => {
        $(
            #[$doc]
            pub const $ident: &'static str = $value;
        )*
    };
}

impl DefaultRoleId {
    def!(
        ALL, "1", /// 全体
        ADMIN, "2", /// 管理员
        OWNER, "4", /// 群主/创建者
        CHANNEL_ADMIN, "5", /// 子频道管理员
    );
}
