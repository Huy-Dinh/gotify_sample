use futures::{executor};
use grpc_server::start_server;
use log::{debug, error};
use monitor::{
    top_news_monitor::{
        config::ParserType,
        news_api_fetcher::NewsApiFetcher,
        news_scraper_fetcher::{soha_parser, vnexpress_parser, NewsScraperFetcher},
        TopNewsMonitor, persistence,
    },
    MonitorNotification,
};

use monitor::top_news_monitor::config::{MonitorConfiguration, MonitorType};

use url::Url;

use std::{
    sync::mpsc::{channel, Sender},
};

mod monitor;
mod notification_sender;
mod grpc_server;

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

    let persistence = persistence::TopNewsMonitorPersistence::new();

    let _top_news_monitors: Vec<TopNewsMonitor> = persistence.get_configurations()
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

    let grpc_server = grpc_server::GrpcMonitorServer::new(persistence);
    let server_task = start_server(50051, grpc_server);

    server_task.await;
    notification_receiver_task.await.unwrap();
}
