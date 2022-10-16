use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Channel {
    #[serde_as(as = "DisplayFromStr")]
    /// 子频道 id
    pub id: u64,
    #[serde_as(as = "DisplayFromStr")]
    /// 频道 id
    pub guild_id: u64,
    /// 子频道名
    pub name: String,
    /// 子频道类型 ChannelType
    pub r#type: ChannelType,
    /// 子频道子类型 ChannelSubType
    pub sub_type: ChannelSubType,
    /// 排序值，具体请参考 有关 position 的说明
    pub position: i32,
    /// 所属分组 id，仅对子频道有效，对 子频道分组（ChannelType=4） 无效
    pub parent_id: String,
    #[serde_as(as = "DisplayFromStr")]
    /// 创建人 id
    pub owner_id: u64,
    /// 子频道私密类型 PrivateType
    pub private_type: PrivateType,
    /// 子频道发言权限 SpeakPermission
    pub speak_permission: SpeakPermission,
    #[serde(skip_serializing_if ="Option::is_none")]
    /// 用于标识应用子频道应用类型，仅应用子频道时会使用该字段，具体定义请参考 应用子频道的应用类型
    pub application_id: Option<String>,
    /// 用户拥有的子频道权限 Permissions
    pub permissions: String,
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[non_exhaustive]
#[repr(i32)]
pub enum ChannelType {
    /// 文字子频道
    Text = 0,
    /// 语音子频道
    Audio = 2,
    /// 子频道分组
    Group = 4,
    /// 直播子频道
    Live = 10005,
    /// 应用子频道
    Application = 10006,
    /// 论坛子频道
    Forum = 10007,
    /// 尚未支持的
    #[serde(other)]
    Unsupported = i32::MAX
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[non_exhaustive]
#[repr(i32)]
pub enum ChannelSubType {
    ///闲聊
    Chat = 0,
    ///公告
    Notice = 1,
    ///攻略
    Walkthrough = 2,
    ///开黑
    TeamUp = 3,
    /// 尚未支持的
    #[serde(other)]
    Unsupported = i32::MAX
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(i32)]
pub enum PrivateType {
    /// 公开频道
    Public = 0,
    /// 群主管理员可见
    OnlyAdmin = 1,
    /// 群主管理员+指定成员，可使用 修改子频道权限接口 指定成员
    CertainMembers = 2,
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(i32)]
pub enum SpeakPermission {
    /// 无效类型
    Invalid = 0,
    /// 所有人
    All = 1,
    /// 群主管理员+指定成员，可使用 修改子频道权限接口 指定成员
    CertainMembers = 2,
}

#[allow(non_upper_case_globals)]
pub mod application_id {
    macro_rules! def {
        ($description:ident, $value:expr) => {
            pub const $description: &'static str = $value;
        };
    }
    def!(王者开黑大厅, "1000000");
    def!(互动小游戏, "1000001");
    def!(腾讯投票, "1000010");
    def!(飞车开黑大厅, "1000051");
    def!(日程提醒, "1000050");
    def!(CoDM开黑大厅, "1000070");
    def!(和平精英开黑大厅, "1010000");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::*;
    #[test]
    fn test_json() {
        let json_str = include_str!("./testdata/channel.json");
        let v = from_str::<Vec<Channel>>(json_str);
        v.unwrap();
    }
}