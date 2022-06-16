use super::MonitorNotification;
use chrono::prelude::*;
use log::{error, warn};
use std::{
    error::Error,
    fmt::{self, Display},
    sync::mpsc::Sender,
    time::Duration,
};

const APP_TOKEN: &'static str = "A7opbHJXd4qnc7Z";
const API_KEY: &'static str = "db957bc6a67148abbb9a6e35402123e3";

pub struct TopNewsMonitor {
    api_key: &'static str,
}

#[derive(Debug, Clone)]
struct NoArticleError;

impl Error for NoArticleError {}

impl Display for NoArticleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No Article array found")
    }
}

#[derive(Debug, Clone)]
struct ResponseParsingFailed;

impl Error for ResponseParsingFailed {}

impl Display for ResponseParsingFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse response")
    }
}

async fn get_top_news(
    api_key: &str,
    country: &str,
    topic: &str,
) -> Result<Option<(String, String)>, Box<dyn Error>> {
    let json_body = reqwest::Client::new()
        .get("https://newsapi.org/v2/top-headlines")
        .query(&[("country", country), ("apiKey", api_key), ("q", topic)])
        .header("User-Agent", "Cool guy")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let articles = match json_body["articles"].as_array() {
        None => return Err(NoArticleError.into()),
        Some(articles_array) => articles_array,
    };

    if articles.len() == 0 {
        return Ok(None);
    }

    let first_article = match articles[0].as_object() {
        None => return Err(ResponseParsingFailed.into()),
        Some(first_article_object) => first_article_object,
    };

    let title = match first_article["title"].as_str() {
        None => return Err(ResponseParsingFailed.into()),
        Some(title_str) => title_str,
    };

    let source = match first_article["source"].as_object() {
        None => "No Source",
        Some(object) => object["name"].as_str().unwrap_or("No Source"),
    };

    Ok(Some((title.to_string(), source.to_string())))
}

impl TopNewsMonitor {
    pub fn new(optional_api_key: Option<&'static str>) -> TopNewsMonitor {
        let mut this_api_key = API_KEY;
        if let Some(api_key) = optional_api_key {
            this_api_key = api_key;
        }

        TopNewsMonitor {
            api_key: this_api_key,
        }
    }

    pub fn start(
        &self,
        sender: Sender<MonitorNotification>,
        country: &'static str,
        topic: &'static str,
        interval: u64,
    ) {
        let api_key = self.api_key;
        let running_fn = async move {
            loop {
                let current_time = Local::now();

                // Only notify during day time
                if current_time.hour() >= 8 && current_time.hour() <= 21 {
                    let top_news_result: Option<(String, String)> =
                        match get_top_news(api_key, country, topic).await {
                            Err(e) => {
                                error!("{}", e);
                                None
                            }
                            Ok(result) => result,
                        };

                    if let Some((news_title, news_author)) = top_news_result {
                        let notification = MonitorNotification {
                            app_token: APP_TOKEN,
                            title: news_author,
                            message: news_title,
                        };

                        match sender.send(notification) {
                            Err(e) => {
                                error!("Error sending from top news monitor {:?}", e);
                            }
                            Ok(_) => {}
                        }
                    } else {
                        warn!(
                            "Empty News result for topic: \"{}\" in country: \"{}\"",
                            topic, country
                        );
                    }
                } else {
                    warn!("Current time is {}, you're probably sleeping, not gonna wake you up", current_time);
                }

                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        };

        tokio::spawn(running_fn);
    }
}
