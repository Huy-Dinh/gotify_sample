use tokio::sync::mpsc::Sender;

use crate::{
    grpc_server::monitor_grpc_service as grpc,
    monitor::{
        top_news_monitor::{
            config,
            news_api_fetcher::NewsApiFetcher,
            news_scraper_fetcher::{soha_parser, vnexpress_parser, NewsScraperFetcher},
            TopNewsMonitor,
        },
        MonitorNotification,
    },
};
use std::time::Duration;

pub fn create_monitor(
    sender: Sender<MonitorNotification>,
    config: &config::MonitorConfiguration,
) -> TopNewsMonitor {
    match &config.monitor_type {
        config::MonitorType::ApiMonitor {
            api_key,
            country,
            topic,
        } => {
            let fetcher = NewsApiFetcher::new(api_key.clone(), country, topic.clone());
            TopNewsMonitor::new(sender, fetcher, config.interval)
        }
        config::MonitorType::ScraperMonitor {
            name,
            url,
            parser_type,
        } => {
            let parser = match parser_type {
                config::ParserType::Soha => soha_parser::parse_soha,
                config::ParserType::VnExpress => vnexpress_parser::parse_vnexpress,
            };
            TopNewsMonitor::new(
                sender,
                NewsScraperFetcher::new(name, url, parser),
                config.interval,
            )
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

impl From<&config::MonitorConfiguration> for grpc::MonitorConfiguration {
    fn from(monitor_config: &config::MonitorConfiguration) -> Self {
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
                    api_key: api_key.clone(),
                    country: country.clone(),
                    topic: topic.clone(),
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

        grpc::MonitorConfiguration {
            interval_in_seconds: monitor_config.interval.as_secs(),
            monitor_type,
            news_api_configuration,
            scraper_configuration,
        }
    }
}

impl From<&grpc::MonitorConfiguration> for config::MonitorConfiguration {
    fn from(config: &grpc::MonitorConfiguration) -> Self {
        let monitor_type: config::MonitorType;

        if config.monitor_type == grpc::MonitorType::NewsApi as i32 {
            let api_config = config.news_api_configuration.as_ref().unwrap();

            monitor_type = config::MonitorType::ApiMonitor {
                api_key: api_config.api_key.clone(),
                country: api_config.country.clone(),
                topic: api_config.topic.clone(),
            };
        } else {
            // must be Scraper
            let scraper_config = config.scraper_configuration.as_ref().unwrap();

            let parser_type: config::ParserType;

            if scraper_config.parser_type == grpc::ParserType::Soha as i32 {
                parser_type = config::ParserType::Soha
            } else {
                parser_type = config::ParserType::VnExpress
            }

            monitor_type = config::MonitorType::ScraperMonitor {
                url: scraper_config.url.clone(),
                name: scraper_config.name.clone(),
                parser_type,
            }
        }

        config::MonitorConfiguration {
            interval: Duration::from_secs(config.interval_in_seconds),
            monitor_type: monitor_type,
        }
    }
}
