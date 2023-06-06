use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InlineKeyboard {
    /// 数组的一项代表消息按钮组件的一行,最多含有 5 行
    rows: Vec<InlineKeyboardRow>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InlineKeyboardRow {
    /// 数组的一项代表一个按钮，每个 InlineKeyboardRow 最多含有 5 个 Button
    buttons: Vec<Button>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Button {
    /// 按钮 id
    pub id: String,
    /// 按纽渲染展示对象 用于设定按钮的显示效果
    pub render_data: RenderData,
    /// 该按纽操作相关字段 用于设定按钮点击后的操作
    pub action: Action,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RenderData {
    /// 按纽上的文字
    pub label: String,
    /// 点击后按纽上文字
    pub visited_label: String,
    #[serde(default = "RenderStyle::gray_line_box")]
    /// 按钮样式，参考 RenderStyle
    pub style: RenderStyle,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[repr(u8)]
#[non_exhaustive]
pub enum RenderStyle {
    GrayLineBox = 0,
    BlueLineBox = 1,
}

impl RenderStyle {
    const fn gray_line_box() -> Self {
        RenderStyle::GrayLineBox
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    /// 操作类型，参考 ActionType
    pub r#type: i32,
    /// 用于设定操作按钮所需的权限
    pub permission: Permission,
    /// 可点击的次数, 默认不限
    pub click_limit: i32,
    /// 操作相关数据
    pub data: String,
    /// false:不弹出子频道选择器 true:弹出子频道选择器
    pub at_bot_show_channel_list: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[repr(i32)]
#[serde(from = "PermissionSerde", into = "PermissionSerde")]
pub enum Permission {
    SpecifyUser { specify_user_ids: Vec<u64> },
    OnlyAdmin,
    All,
    CertainRole { specify_role_ids: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct PermissionSerde {
    r#type: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    specify_user_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    specify_role_ids: Option<Vec<String>>,
}

impl From<PermissionSerde> for Permission {
    fn from(permission_serde: PermissionSerde) -> Self {
        match permission_serde.r#type {
            0 => Permission::SpecifyUser {
                specify_user_ids: permission_serde
                    .specify_user_ids
                    .unwrap_or_default()
                    .iter()
                    .map(|x| x.parse::<u64>().expect("user id must be u64, if crash here, please report(but it's tencent's bug)"))
                    .collect(),
            },
            1 => Permission::OnlyAdmin,
            2 => Permission::All,
            3 => Permission::CertainRole {
                specify_role_ids: permission_serde.specify_role_ids.unwrap_or_default(),
            },
            premission_type => panic!("cannot parse permission type: {premission_type}"),
        }
    }
}

impl From<Permission> for PermissionSerde {
    fn from(permisson: Permission) -> Self {
        match permisson {
            Permission::SpecifyUser { specify_user_ids } => PermissionSerde {
                r#type: 0,
                specify_user_ids: Some(specify_user_ids.iter().map(|x| x.to_string()).collect()),
                ..Default::default()
            },
            Permission::OnlyAdmin => PermissionSerde {
                r#type: 1,
                ..Default::default()
            },
            Permission::All => PermissionSerde {
                r#type: 2,
                ..Default::default()
            },
            Permission::CertainRole { specify_role_ids } => PermissionSerde {
                r#type: 3,
                specify_role_ids: Some(specify_role_ids),
                ..Default::default()
            },
        }
    }
}