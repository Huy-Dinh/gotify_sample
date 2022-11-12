use futures::executor;
use log::{debug, error};
use monitor::{
    top_news_monitor::{
        config::ParserType,
        news_api_fetcher::NewsApiFetcher,
        news_scraper_fetcher::{soha_parser, vnexpress_parser, NewsScraperFetcher},
        TopNewsMonitor,
    },
    MonitorNotification,
};

use monitor::top_news_monitor::config::{MonitorConfiguration, MonitorType};

use url::Url;

use std::{
    sync::mpsc::{channel, Sender},
    time::Duration,
};

mod monitor;
mod notification_sender;

const BASE_URL_STRING: &str = "https://gotify.van-ngo.com";

fn create_monitor(
    sender: Sender<MonitorNotification>,
    config: &MonitorConfiguration,
) -> TopNewsMonitor {
    match &config.monitor_type {
        MonitorType::ApiMonitor {
            api_key,
            country,
            topic,
        } => {
            let fetcher = NewsApiFetcher::new(api_key.clone(), country, topic.clone());
            TopNewsMonitor::new(sender, fetcher, config.interval)
        }
        MonitorType::ScraperMonitor {
            name,
            url,
            parser_type,
        } => {
            let parser = match parser_type {
                ParserType::Soha => soha_parser::parse_soha,
                ParserType::VnExpress => vnexpress_parser::parse_vnexpress,
            };
            TopNewsMonitor::new(
                sender,
                NewsScraperFetcher::new(name, url, parser),
                config.interval,
            )
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let base_url = Url::parse(BASE_URL_STRING).expect("Failed to parse the base url");

    let (sender, receiver) = channel::<monitor::MonitorNotification>();

    let monitor_configs = [
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
    ];

    let _top_news_monitors: Vec<TopNewsMonitor> = monitor_configs
        .iter()
        .map(|config| create_monitor(sender.clone(), config))
        .collect();

    let notification_receiver_task = tokio::task::spawn_blocking(move || {
        while let Ok(msg) = receiver.recv() {
            let send_result = executor::block_on(notification_sender::send_notification(
                &base_url,
                msg.app_token,
                &msg.title,
                &msg.message,
                &msg.image_url,
                &msg.article_link,
                10,
            ));
            match send_result {
                Err(e) => {
                    error!("{}", e);
                }
                Ok(()) => {
                    debug!("Sent: {:?}", &msg);
                }
            }
        }
    });

    notification_receiver_task.await.unwrap();
}
