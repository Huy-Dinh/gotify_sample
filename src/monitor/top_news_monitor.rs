use super::MonitorNotification;
use chrono::prelude::*;
use log::{error, warn};
use std::{
    error::Error,
    fmt::{self, Display},
    sync::mpsc::Sender,
    time::Duration,
};

use tokio::task::JoinHandle;

const APP_TOKEN: &'static str = "A7opbHJXd4qnc7Z";
const API_KEY: &'static str = "db957bc6a67148abbb9a6e35402123e3";

pub struct TopNewsMonitor {
    api_key: &'static str,
    optional_task_handle: Option<JoinHandle<()>>,
    country: String,
    topic: Option<String>,
    interval: u64,
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
    topic: &Option<String>,
) -> Result<Option<(String, String)>, Box<dyn Error>> {
    let mut request_builder = reqwest::Client::new()
        .get("https://newsapi.org/v2/top-headlines")
        .query(&[("country", country), ("apiKey", api_key)])
        .header("User-Agent", "Cool guy");

    if let Some(topic_string) = topic {
        request_builder = request_builder.query(&[("q", topic_string)])
    }

    let json_body = request_builder
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
    pub fn new(
        optional_api_key: Option<&'static str>,
        country: &str,
        optional_topic: Option<&str>,
        interval: u64,
    ) -> TopNewsMonitor {
        let mut this_api_key = API_KEY;
        if let Some(api_key) = optional_api_key {
            this_api_key = api_key;
        }

        let mut this_topic: Option<String> = None;
        if let Some(topic) = optional_topic {
            this_topic = Some(topic.to_string());
        }

        TopNewsMonitor {
            api_key: this_api_key,
            optional_task_handle: None,
            country: country.to_string(),
            topic: this_topic,
            interval: interval,
        }
    }

    pub fn start(&mut self, sender: Sender<MonitorNotification>) {
        let api_key = self.api_key;
        let country = self.country.clone();
        let topic = self.topic.clone();
        let interval = self.interval;

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
            
                let top_news_result = match get_top_news(api_key, &country, &topic).await {
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                    Ok(result) => result,
                };
            
                if let Some((news_title, news_author)) = top_news_result {
                    let notification = MonitorNotification {
                        app_token: APP_TOKEN,
                        title: news_author,
                        message: news_title,
                    };
            
                    if let Err(e) = sender.send(notification) {
                        error!("Error sending from top news monitor {:?}", e);
                        continue;
                    }
                } else {
                    warn!(
                        "Empty News result for topic: \"{}\" in country: \"{}\"",
                        topic.clone().unwrap_or("".to_string()),
                        country
                    );
                }
            }
        };

        // Save the task handle so we can stop it later
        self.optional_task_handle = Some(tokio::spawn(running_fn));
    }

    pub fn stop(&self) {
        if let Some(task_handle) = &self.optional_task_handle {
            task_handle.abort();
        }
    }
}
