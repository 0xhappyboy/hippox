#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverSignalStatus {
    Pause,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverSignal {
    status: DriverSignalStatus,
    msg: Option<String>,
}
