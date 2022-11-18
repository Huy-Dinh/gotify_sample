use std::time::Duration;

pub struct MonitorConfiguration {
    pub interval: Duration,
    pub monitor_type: MonitorType,
}

pub enum ParserType {
    Soha,
    VnExpress,
}
pub enum MonitorType {
    ApiMonitor {
        api_key: Option<String>,
        country: String,
        topic: Option<String>,
    },
    ScraperMonitor {
        url: String,
        name: String,
        parser_type: ParserType,
    },
}
