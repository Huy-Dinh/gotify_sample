use super::MonitorNotification;
use async_trait::async_trait;
use log::{debug, error};
use std::{
    error::Error,
    fmt::{self, Display},
    sync::{mpsc::Sender, Arc},
    time::Duration,
};

use tokio::{sync::Mutex, task::JoinHandle, time::Instant};

pub mod news_api_fetcher;
pub mod soha_scrape_fetcher;
pub mod vnexpress_scrape_fetcher;

const APP_TOKEN: &'static str = "A7opbHJXd4qnc7Z";

pub struct NewsInfo {
    title: String,
    source: String,
    image_url: Option<String>,
    article_url: Option<String>
}

#[async_trait]
pub trait NewsFetcher {
    async fn fetch_news(
        &mut self,
    ) -> Result<Option<NewsInfo>, Box<dyn Error>>;
}

pub struct TopNewsMonitor {
    fetcher: Arc<Mutex<dyn NewsFetcher + Sync + Send>>,
    task_handle: Option<JoinHandle<()>>,
    interval: u64,
}

#[derive(Debug, Clone)]
struct ResponseParsingFailed;

impl Error for ResponseParsingFailed {}

impl Display for ResponseParsingFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse response")
    }
}

impl TopNewsMonitor {
    pub fn new(
        fetcher: Arc<Mutex<dyn NewsFetcher + Sync + Send>>,
        interval: u64,
    ) -> TopNewsMonitor {
        TopNewsMonitor {
            fetcher: fetcher,
            task_handle: None,
            interval: interval,
        }
    }

    pub fn start(&mut self, sender: Sender<MonitorNotification>) {
        let interval = self.interval;
        let fetcher = self.fetcher.clone();
        let running_fn = async move {
            let mut next_wake_instant = Instant::now();

            loop {
                tokio::time::sleep_until(next_wake_instant).await;
                next_wake_instant = Instant::now() + Duration::from_secs(interval);

                let top_news_result = match fetcher.lock().await.fetch_news().await {
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                    Ok(result) => result,
                };

                if let Some(news_info) = top_news_result {
                    let notification = MonitorNotification {
                        app_token: APP_TOKEN,
                        title: news_info.source,
                        message: news_info.title,
                        image_url: news_info.image_url,
                        article_link: news_info.article_url,
                    };

                    if let Err(e) = sender.send(notification) {
                        error!("Error sending from top news monitor {:?}", e);
                        continue;
                    }
                } else {
                    debug!("Empty News result");
                }
            }
        };

        // Save the task handle so we can stop it later
        self.task_handle = Some(tokio::spawn(running_fn));
    }
}

impl Drop for TopNewsMonitor {
    fn drop(&mut self) {
        if let Some(task_handle) = &self.task_handle {
            task_handle.abort();
            debug!("Aborted running task on drop");
        }
    }
}
