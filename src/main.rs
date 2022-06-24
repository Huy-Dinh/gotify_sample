use log::{error, info};
use monitor::top_news_monitor::{
    news_api_fetcher::NewsApiFetcher, soha_scrape_fetcher::SohaScrapeFetcher, TopNewsMonitor,
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

    let mut top_news_monitors = vec![
        TopNewsMonitor::new(
            Arc::new(Mutex::new(NewsApiFetcher::new(None, "de", None))),
            7200,
        ),
        TopNewsMonitor::new(Arc::new(Mutex::new(SohaScrapeFetcher::new("quoc-te.htm"))), 60),
        TopNewsMonitor::new(Arc::new(Mutex::new(SohaScrapeFetcher::new("cong-nghe.htm"))), 60),
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
                info!("Sent: {:?}", &msg);
            }
        }
    }
}
