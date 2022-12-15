use crate::{
    grpc_server::monitor_grpc_service as grpc,
    monitor::top_news_monitor::{
        config,
        news_api_fetcher::NewsApiFetcher,
        news_scraper_fetcher::{soha_parser, vnexpress_parser, NewsScraperFetcher},
        NewsFetcher,
    },
};
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

pub fn create_fetcher(
    config: &config::TopNewsMonitorDatabaseEntry,
) -> Arc<dyn NewsFetcher + Send + Sync + 'static> {
    match &config.monitor_type {
        config::MonitorType::ApiMonitor {
            api_key,
            country,
            topic,
        } => Arc::new(NewsApiFetcher::new(api_key.clone(), country, topic.clone())),
        config::MonitorType::ScraperMonitor {
            url,
            name,
            parser_type,
        } => {
            let parser = match parser_type {
                config::ParserType::Soha => soha_parser::parse_soha,
                config::ParserType::VnExpress => vnexpress_parser::parse_vnexpress,
            };
            Arc::new(NewsScraperFetcher::new(name, url, parser))
        }
    }
}

impl From<&config::ParserType> for grpc::ParserType {
    fn from(parser_type: &config::ParserType) -> Self {
        match parser_type {
            config::ParserType::Soha => grpc::ParserType::Soha,
            config::ParserType::VnExpress => grpc::ParserType::Vnexpress,
        }
    }
}

impl From<&config::TopNewsMonitorDatabaseEntry> for grpc::MonitorConfiguration {
    fn from(monitor_config: &config::TopNewsMonitorDatabaseEntry) -> Self {
        let mut news_api_configuration: Option<grpc::NewsApiConfiguration> = None;
        let mut scraper_configuration: Option<grpc::ScraperApiConfiguration> = None;
        let monitor_type: i32;

        match &monitor_config.monitor_type {
            config::MonitorType::ApiMonitor {
                api_key,
                country,
                topic,
            } => {
                monitor_type = grpc::MonitorType::NewsApi as i32;
                news_api_configuration = Some(grpc::NewsApiConfiguration {
                    api_key: api_key.clone().unwrap_or_else(|| String::from("")),
                    country: country.clone(),
                    topic: topic.clone().unwrap_or_else(|| String::from("")),
                });
            }
            config::MonitorType::ScraperMonitor {
                url,
                name,
                parser_type,
            } => {
                monitor_type = grpc::MonitorType::WebScraper as i32;

                scraper_configuration = Some(grpc::ScraperApiConfiguration {
                    url: url.clone(),
                    name: name.clone(),
                    parser_type: grpc::ParserType::from(parser_type) as i32,
                });
            }
        }

        let state = match monitor_config.state {
            config::State::Paused => grpc::State::Paused as i32,
            config::State::Running => grpc::State::Running as i32,
        };

        grpc::MonitorConfiguration {
            interval_in_seconds: monitor_config.interval.as_secs(),
            monitor_type,
            news_api_configuration,
            scraper_configuration,
            state,
        }
    }
}

impl From<&grpc::MonitorConfiguration> for config::TopNewsMonitorDatabaseEntry {
    fn from(config: &grpc::MonitorConfiguration) -> Self {
        let monitor_type = if config.monitor_type == grpc::MonitorType::NewsApi as i32 {
            let api_config = config.news_api_configuration.as_ref().unwrap();

            config::MonitorType::ApiMonitor {
                api_key: Some(api_config.api_key.clone()),
                country: api_config.country.clone(),
                topic: Some(api_config.topic.clone()),
            }
        } else {
            // must be Scraper
            let scraper_config = config.scraper_configuration.as_ref().unwrap();

            let parser_type = if scraper_config.parser_type == grpc::ParserType::Soha as i32 {
                config::ParserType::Soha
            } else {
                config::ParserType::VnExpress
            };

            config::MonitorType::ScraperMonitor {
                url: scraper_config.url.clone(),
                name: scraper_config.name.clone(),
                parser_type,
            }
        };

        let state = if config.state == grpc::State::Paused as i32 {
            config::State::Paused
        } else {
            config::State::Running
        };

        config::TopNewsMonitorDatabaseEntry {
            id: Uuid::new_v4(),
            interval: Duration::from_secs(config.interval_in_seconds),
            monitor_type,
            state,
        }
    }
}

impl From<&config::TopNewsMonitorDatabaseEntry> for grpc::MonitorEntry {
    fn from(db_entry: &config::TopNewsMonitorDatabaseEntry) -> Self {
        Self {
            id: db_entry.id.to_string(),
            configuration: Some(grpc::MonitorConfiguration::from(db_entry)),
        }
    }
}
