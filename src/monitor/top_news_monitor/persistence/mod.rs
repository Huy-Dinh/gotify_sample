use super::config::{MonitorType, ParserType, State, TopNewsMonitorDatabaseEntry};
use std::time::Duration;
use uuid::Uuid;

pub struct TopNewsMonitorPersistence {
    monitor_configurations: Vec<TopNewsMonitorDatabaseEntry>,
}

impl TopNewsMonitorPersistence {
    pub fn new() -> TopNewsMonitorPersistence {
        TopNewsMonitorPersistence {
            monitor_configurations: Vec::from([
                TopNewsMonitorDatabaseEntry {
                    id: Uuid::new_v4(),
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://soha.vn/"),
                        name: String::from("Soha"),
                        parser_type: ParserType::Soha,
                    },
                    state: State::Paused,
                },
                TopNewsMonitorDatabaseEntry {
                    id: Uuid::new_v4(),
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://soha.vn/quoc-te.htm"),
                        name: String::from("Soha quốc tế"),
                        parser_type: ParserType::Soha,
                    },
                    state: State::Running,
                },
                TopNewsMonitorDatabaseEntry {
                    id: Uuid::new_v4(),
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://soha.vn/cong-nghe.htm"),
                        name: String::from("Soha công nghệ"),
                        parser_type: ParserType::Soha,
                    },
                    state: State::Running,
                },
                TopNewsMonitorDatabaseEntry {
                    id: Uuid::new_v4(),
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://vnexpress.net/"),
                        name: String::from("VnExpress"),
                        parser_type: ParserType::VnExpress,
                    },
                    state: State::Running,
                },
                TopNewsMonitorDatabaseEntry {
                    id: Uuid::new_v4(),
                    interval: Duration::from_secs(1800),
                    monitor_type: MonitorType::ScraperMonitor {
                        url: String::from("https://vnexpress.net/the-gioi"),
                        name: String::from("VnExpress quốc tế"),
                        parser_type: ParserType::VnExpress,
                    },
                    state: State::Running,
                },
            ]),
        }
    }

    pub fn get_configurations(&self) -> &Vec<TopNewsMonitorDatabaseEntry> {
        &self.monitor_configurations
    }

    pub fn add_configuration(
        &mut self,
        new_configuration: TopNewsMonitorDatabaseEntry,
    ) -> Result<(), ()> {
        self.monitor_configurations.push(new_configuration);
        Ok(())
    }

    pub fn remove_configuration(&mut self, id: &Uuid) -> Result<(), ()> {
        self.monitor_configurations
            .retain(|monitor| monitor.id != *id);
        Ok(())
    }
}
