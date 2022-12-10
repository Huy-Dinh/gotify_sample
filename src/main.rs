use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use grpc_server::start_server;
use log::{debug, error, info, warn};
use monitor::{
    top_news_monitor::{
        persistence::{self, TopNewsMonitorPersistence},
        run_monitor, NewsFetcher,
    },
    MonitorNotification,
};

use tokio::{
    sync::mpsc::{channel, Sender},
    time::Instant,
};
use url::Url;
use uuid::Uuid;

mod grpc_server;
mod helper;
mod monitor;
mod notification_sender;

const BASE_URL_STRING: &str = "https://gotify.van-ngo.com";

async fn news_fetching_task(
    sender: Sender<MonitorNotification>,
    persistence: Arc<Mutex<TopNewsMonitorPersistence>>,
) {
    let mut next_wake_instant = Instant::now();
    let mut fetcher_map: HashMap<Uuid, Arc<dyn NewsFetcher + Send + Sync + 'static>> =
        HashMap::new();

    let mut accumulated_time = Duration::from_micros(0);

    info!("Fetching task entered");

    loop {
        info!("Fetching task entered looop");
        tokio::time::sleep_until(next_wake_instant).await;
        next_wake_instant = Instant::now() + Duration::from_secs(30);

        persistence
            .lock()
            .unwrap()
            .get_configurations()
            .iter()
            .for_each(|config| {
                if accumulated_time.as_secs() % config.interval.as_secs() != 0 {
                    warn!("Time doesnt match");
                    return;
                }

                warn!("Time match! {:?}", &accumulated_time);

                // If we already have a fetcher for this configuration
                if !fetcher_map.contains_key(&config.id) {
                    warn!("Missing fetcher, adding new");
                    fetcher_map.insert(config.id, helper::create_fetcher(config));
                }

                let fetcher = fetcher_map.get(&config.id).unwrap().clone();
                let sender_clone = sender.clone();

                tokio::spawn(async {
                    run_monitor(sender_clone, fetcher);
                });
            });
        accumulated_time += Duration::from_secs(30);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let base_url = Url::parse(BASE_URL_STRING).expect("Failed to parse the base url");

    let (sender, mut receiver) = channel::<monitor::MonitorNotification>(64);

    let persistence = Arc::new(Mutex::new(persistence::TopNewsMonitorPersistence::new()));

    let news_fetching_task = tokio::task::spawn(news_fetching_task(sender, persistence.clone()));

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

    let grpc_server = grpc_server::GrpcMonitorServer::new(persistence.clone());
    let server_task = start_server(50051, grpc_server);

    server_task.await;
    notification_receiver_task.await.unwrap();
    news_fetching_task.await.unwrap();
}
