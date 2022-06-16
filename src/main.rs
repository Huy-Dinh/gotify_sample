use monitor::top_news_monitor::TopNewsMonitor;
use url::Url;
use log::{info, error};

use std::sync::mpsc::channel;

mod monitor;
mod notification_sender;

const BASE_URL_STRING: &'static str = "https://gotify.van-ngo.com";

// tokio let's us use "async" on our main function
#[tokio::main]
async fn main() {
    env_logger::init();

    let base_url = Url::parse(BASE_URL_STRING).expect("Failed to parse the base url");

    let (sender, receiver) = channel::<monitor::MonitorNotification>();

    TopNewsMonitor::new(None).start(sender.clone(), "us", "bitcoin", 3600);
    TopNewsMonitor::new(None).start(sender.clone(), "us", "recession", 7200);
    TopNewsMonitor::new(None).start(sender.clone(), "de", "", 7200);

    while let Ok(msg) = receiver.recv() {
        match notification_sender::send_notification(
            &base_url,
            msg.app_token,
            &msg.title,
            &msg.message,
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
