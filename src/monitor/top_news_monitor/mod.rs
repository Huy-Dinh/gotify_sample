use super::MonitorNotification;
use async_trait::async_trait;
use log::{debug, error};
use std::{
    error::Error,
    fmt::{self, Display},
    sync::{mpsc::Sender},
    time::Duration,
};

use tokio::{task::JoinHandle, time::Instant};

pub mod persistence;
pub mod config;
pub mod news_api_fetcher;
pub mod news_scraper_fetcher;

const APP_TOKEN: &str = "A7opbHJXd4qnc7Z";

pub struct NewsInfo {
    title: String,
    source: String,
    image_url: Option<String>,
    article_url: Option<String>
}

#[async_trait]
pub trait NewsFetcher {
    async fn fetch_news(
        &self,
    ) -> Result<Option<NewsInfo>, Box<dyn Error>>;
}

pub struct TopNewsMonitor {
    task_handle: JoinHandle<()>
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
        sender: Sender<MonitorNotification>,
        fetcher: impl NewsFetcher + Sync + Send + 'static,
        interval: Duration,
    ) -> TopNewsMonitor {

        let running_fn = async move {
            let mut next_wake_instant = Instant::now();

            loop {
                tokio::time::sleep_until(next_wake_instant).await;
                next_wake_instant = Instant::now() + interval;

                let top_news_result = fetcher.fetch_news().await;
                
                let top_news_result = match top_news_result {
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

        TopNewsMonitor {
            task_handle: tokio::spawn(running_fn),
        }
    }

}

impl Drop for TopNewsMonitor {
    fn drop(&mut self) {
        self.task_handle.abort();
        debug!("Aborted running task on drop");
    }
}
