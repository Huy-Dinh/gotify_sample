use std::time::Duration;

use super::config::{MonitorConfiguration, MonitorType, ParserType};

pub struct TopNewsMonitorPersistence {
    monitor_configurations: Vec<MonitorConfiguration>,
}

impl TopNewsMonitorPersistence {
    pub fn new() -> TopNewsMonitorPersistence {
        TopNewsMonitorPersistence {
            monitor_configurations: Vec::from([
                MonitorConfiguration {
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://soha.vn/"),
                        name: String::from("Soha"),
                        parser_type: ParserType::Soha,
                    },
                },
                MonitorConfiguration {
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://soha.vn/quoc-te.htm"),
                        name: String::from("Soha quốc tế"),
                        parser_type: ParserType::Soha,
                    },
                },
                MonitorConfiguration {
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://soha.vn/cong-nghe.htm"),
                        name: String::from("Soha công nghệ"),
                        parser_type: ParserType::Soha,
                    },
                },
                MonitorConfiguration {
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://vnexpress.net/"),
                        name: String::from("VnExpress"),
                        parser_type: ParserType::VnExpress,
                    },
                },
                MonitorConfiguration {
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://vnexpress.net/the-gioi"),
                        name: String::from("VnExpress quốc tế"),
                        parser_type: ParserType::VnExpress,
                    },
                },
            ]),
        }
    }

    pub fn get_configurations(&self) -> &Vec<MonitorConfiguration> {
        &self.monitor_configurations
    }

    #[allow(dead_code)]
    fn add_configuration(&mut self, new_configuration: MonitorConfiguration) -> Result<(), ()> {
        self.monitor_configurations.push(new_configuration);
        Ok(())
    }

    pub fn remove_configuration(&mut self, index: usize) -> Result<(), ()> {
        self.monitor_configurations.remove(index);
        Ok(())
    }
}
