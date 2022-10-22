use log::{debug, error};
use monitor::top_news_monitor::{
    soha_scrape_fetcher::SohaScrapeFetcher, vnexpress_scrape_fetcher::VnExpressScrapeFetcher,
    TopNewsMonitor,
};
use url::Url;

use tokio::sync::Mutex;

use std::sync::{mpsc::channel, Arc};

mod monitor;
mod notification_sender;

const BASE_URL_STRING: &'static str = "https://gotify.van-ngo.com";

#[tokio::main]
async fn main() {
    env_logger::init();

    let base_url = Url::parse(BASE_URL_STRING).expect("Failed to parse the base url");

    let (sender, receiver) = channel::<monitor::MonitorNotification>();

    let soha_general_monitor = TopNewsMonitor::new(
        Arc::new(Mutex::new(SohaScrapeFetcher::new("chung", None))),
        1800,
    );

    let soha_international_monitor = TopNewsMonitor::new(
        Arc::new(Mutex::new(SohaScrapeFetcher::new(
            "quốc tế",
            Some("quoc-te.htm"),
        ))),
        1800,
    );

    let soha_technology_monitor = TopNewsMonitor::new(
        Arc::new(Mutex::new(SohaScrapeFetcher::new(
            "công nghệ",
            Some("cong-nghe.htm"),
        ))),
        1800,
    );

    let vnexpress_general_monitor = TopNewsMonitor::new(
        Arc::new(Mutex::new(VnExpressScrapeFetcher::new("chung", None))),
        1800,
    );

    let vnexpress_international_monitor = TopNewsMonitor::new(
        Arc::new(Mutex::new(VnExpressScrapeFetcher::new("quốc tế", Some("the-gioi")))),
        3600,
    );

    let mut top_news_monitors = vec![
        soha_general_monitor,
        soha_international_monitor,
        soha_technology_monitor,
        vnexpress_general_monitor,
        vnexpress_international_monitor,
    ];

    for monitor in &mut top_news_monitors {
        monitor.start(sender.clone());
    }

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
