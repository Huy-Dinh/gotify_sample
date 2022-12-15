use super::MonitorNotification;
use async_trait::async_trait;
use log::{debug, error};
use std::{
    error::Error,
    fmt::{self, Display},
    sync::Arc,
};

use tokio::sync::mpsc::Sender;

use anyhow::{Context, Result};

pub mod config;
pub mod news_api_fetcher;
pub mod news_scraper_fetcher;
pub mod persistence;

const APP_TOKEN: &str = "A7opbHJXd4qnc7Z";

pub struct NewsInfo {
    title: String,
    source: String,
    image_url: Option<String>,
    article_url: Option<String>,
}

#[async_trait]
pub trait NewsFetcher {
    async fn fetch_news(&self) -> Result<Option<NewsInfo>>;
}

#[derive(Debug, Clone)]
struct ResponseParsingFailed;

impl Error for ResponseParsingFailed {}

impl Display for ResponseParsingFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse response")
    }
}

pub fn run_monitor(
    sender: Sender<MonitorNotification>,
    fetcher: Arc<dyn NewsFetcher + Sync + Send + 'static>,
) {
    tokio::spawn(async move {
        let top_news_result = fetcher.fetch_news().await;

        let top_news_result = match top_news_result {
            Err(e) => {
                error!("{}", e);
                return;
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

            if let Err(err) = sender
                .send(notification)
                .await
                .context("Failed to send notification to main thread")
            {
                error!("{:?}", err);
            }
        } else {
            debug!("Empty News result");
        }
    });
}
