#[derive(Debug)]
pub struct MonitorNotification {
    pub app_token: &'static str,
    pub title: String,
    pub message: String,
    pub image_url: Option<String>,
    pub article_link: Option<String>
}

pub mod top_news_monitor;