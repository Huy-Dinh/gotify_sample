use futures::executor;
use grpc_server::start_server;
use log::{debug, error};
use monitor::top_news_monitor::{persistence, TopNewsMonitor};

use url::Url;

use std::sync::mpsc::channel;

mod grpc_server;
mod helper;
mod monitor;
mod notification_sender;

const BASE_URL_STRING: &str = "https://gotify.van-ngo.com";

#[tokio::main]
async fn main() {
    env_logger::init();

    let base_url = Url::parse(BASE_URL_STRING).expect("Failed to parse the base url");

    let (sender, receiver) = channel::<monitor::MonitorNotification>();

    let persistence = persistence::TopNewsMonitorPersistence::new();

    let top_news_monitors: Vec<TopNewsMonitor> = persistence
        .get_configurations()
        .iter()
        .map(|config| helper::create_monitor(sender.clone(), config))
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

    let grpc_server = grpc_server::GrpcMonitorServer::new(persistence, top_news_monitors);
    let server_task = start_server(50051, grpc_server);

    server_task.await;
    notification_receiver_task.await.unwrap();
}
