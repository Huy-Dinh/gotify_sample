#[derive(Debug)]
pub struct MonitorNotification {
    pub app_token: &'static str,
    pub title: String,
    pub message: String
}

pub mod cyclic_monitor;