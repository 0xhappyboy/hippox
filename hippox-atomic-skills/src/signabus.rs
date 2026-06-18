#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillSignalStatus {
    Pause,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSignal {
    status: SkillSignalStatus,
    msg: Option<String>,
}
