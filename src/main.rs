use log::{debug, error};
use monitor::top_news_monitor::{
    news_scraper_fetcher::{soha_parser, vnexpress_parser, NewsScraperFetcher},
    TopNewsMonitor,
};
use url::Url;

use std::sync::{mpsc::channel};

mod monitor;
mod notification_sender;

const BASE_URL_STRING: &str = "https://gotify.van-ngo.com";

#[tokio::main]
async fn main() {
    env_logger::init();

    let base_url = Url::parse(BASE_URL_STRING).expect("Failed to parse the base url");

    let (sender, receiver) = channel::<monitor::MonitorNotification>();

    let soha_general_monitor = TopNewsMonitor::new(
        sender.clone(),
        NewsScraperFetcher::new(
            "Soha",
            "https://soha.vn/",
            soha_parser::parse_soha,
        ),
        1800,
    );

    let soha_international_monitor = TopNewsMonitor::new(
        sender.clone(),
        NewsScraperFetcher::new(
            "Soha quốc tế",
            "https://soha.vn/quoc-te.htm",
            soha_parser::parse_soha,
        ),
        1800,
    );

    let soha_technology_monitor = TopNewsMonitor::new(
        sender.clone(),
        NewsScraperFetcher::new(
            "Soha công nghệ",
            "https://soha.vn/cong-nghe.htm",
            soha_parser::parse_soha,
        ),
        1800,
    );

    let vnexpress_general_monitor = TopNewsMonitor::new(
        sender.clone(),
        NewsScraperFetcher::new(
            "VnExpress chung",
            "https://vnexpress.net/",
            vnexpress_parser::parse_vnexpress,
        ),
        1800,
    );

    let vnexpress_international_monitor = TopNewsMonitor::new(
        sender.clone(),
        NewsScraperFetcher::new(
            "VnExpress quốc tế",
            "https://vnexpress.net/the-gioi",
            vnexpress_parser::parse_vnexpress,
        ),
        1800,
    );

    let _top_news_monitors = vec![
        soha_general_monitor,
        soha_international_monitor,
        soha_technology_monitor,
        vnexpress_general_monitor,
        vnexpress_international_monitor,
    ];

    while let Ok(msg) = receiver.recv() {
        match notification_sender::send_notification(
            &base_url,
            msg.app_token,
            &msg.title,
            &msg.message,
            &msg.image_url,
            &msg.article_link,
            10,
        )
        .await
        {
            Err(e) => {
                error!("{}", e);
            }
            Ok(()) => {
                debug!("Sent: {:?}", &msg);
            }
        }
    }
}
