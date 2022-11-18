use grpc_server::start_server;
use log::{debug, error};
use monitor::top_news_monitor::{persistence, TopNewsMonitor};

use tokio::sync::mpsc::channel;
use url::Url;

mod grpc_server;
mod helper;
mod monitor;
mod notification_sender;

const BASE_URL_STRING: &str = "https://gotify.van-ngo.com";

#[tokio::main]
async fn main() {
    env_logger::init();

    let base_url = Url::parse(BASE_URL_STRING).expect("Failed to parse the base url");

    let (sender, mut receiver) = channel::<monitor::MonitorNotification>(64);

    let persistence = persistence::TopNewsMonitorPersistence::new();

    let top_news_monitors: Vec<TopNewsMonitor> = persistence
        .get_configurations()
        .iter()
        .map(|config| helper::create_monitor(sender.clone(), config))
        .collect();

    let notification_receiver_task = tokio::task::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            let send_result = notification_sender::send_notification(
                &base_url,
                msg.app_token,
                &msg.title,
                &msg.message,
                &msg.image_url,
                &msg.article_link,
                10,
            )
            .await;
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

    let grpc_server = grpc_server::GrpcMonitorServer::new(persistence, top_news_monitors, sender);
    let server_task = start_server(50051, grpc_server);

    server_task.await;
    notification_receiver_task.await.unwrap();
}
