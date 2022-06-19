use super::MonitorNotification;
use async_trait::async_trait;
use chrono::prelude::*;
use log::{debug, error, warn};
use std::{
    error::Error,
    fmt::{self, Display},
    sync::{mpsc::Sender, Arc},
    time::Duration,
};

use tokio::task::JoinHandle;

pub mod news_api_fetcher;
pub mod soha_scrape_fetcher;

const APP_TOKEN: &'static str = "A7opbHJXd4qnc7Z";

#[async_trait]
pub trait NewsFetcher {
    async fn fetch_news(&self) -> Result<Option<(String, String, Option<String>, Option<String>)>, Box<dyn Error>>;
}

pub struct TopNewsMonitor {
    fetcher: Arc<dyn NewsFetcher + Sync + Send>,
    optional_task_handle: Option<JoinHandle<()>>,
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
    pub fn new(fetcher: Arc<dyn NewsFetcher + Sync + Send>, interval: u64) -> TopNewsMonitor {
        TopNewsMonitor {
            fetcher: fetcher,
            optional_task_handle: None,
            interval: interval,
        }
    }

    pub fn start(&mut self, sender: Sender<MonitorNotification>) {
        let interval = self.interval;
        let fetcher = self.fetcher.clone();
        let running_fn = async move {
            let mut first_interation = true;

            loop {
                // We don't delay when running the first iteration
                if first_interation {
                    first_interation = false;
                } else {
                    tokio::time::sleep(Duration::from_secs(interval)).await;
                }

                let current_time = Local::now();

                if current_time.hour() < 8 || current_time.hour() > 21 {
                    warn!(
                        "Current time is {}, you're probably sleeping, not gonna wake you up",
                        current_time
                    );
                    continue;
                }

                let top_news_result = match fetcher.fetch_news().await {
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                    Ok(result) => result,
                };

                if let Some((news_title, news_author, image_url, article_link)) = top_news_result {
                    let notification = MonitorNotification {
                        app_token: APP_TOKEN,
                        title: news_author,
                        message: news_title,
                        image_url: image_url,
                        article_link: article_link
                    };

                    if let Err(e) = sender.send(notification) {
                        error!("Error sending from top news monitor {:?}", e);
                        continue;
                    }
                } else {
                    warn!("Empty News result");
                }
            }
        };

        // Save the task handle so we can stop it later
        self.optional_task_handle = Some(tokio::spawn(running_fn));
    }
}

impl Drop for TopNewsMonitor {
    fn drop(&mut self) {
        if let Some(task_handle) = &self.optional_task_handle {
            task_handle.abort();
            debug!("Aborted running task on drop");
        }
    }
}
