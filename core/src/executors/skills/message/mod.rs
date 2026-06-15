pub mod dingding;
pub mod email;
pub mod feishu;
pub mod telegram;
pub mod wecom;

pub use dingding::SendDingDingSkill;
pub use email::SendEmailSkill;
pub use feishu::SendFeishuSkill;
pub use telegram::SendTelegramSkill;
pub use wecom::SendWecomSkill;
