use std::{error::Error, fmt, fmt::Display};

use async_trait::async_trait;

use super::{NewsFetcher, ResponseParsingFailed};

const DEFAULT_API_KEY: &'static str = "db957bc6a67148abbb9a6e35402123e3";

pub struct NewsApiFetcher {
    api_key: String,
    country: String,
    topic: Option<String>,
}

#[derive(Debug, Clone)]
struct NoArticleError;

impl Error for NoArticleError {}

impl Display for NoArticleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No Article array found")
    }
}

impl NewsApiFetcher {
    pub fn new(api_key: Option<String>, country: &str, topic: Option<String>) -> NewsApiFetcher {
        let mut this_api_key = DEFAULT_API_KEY.to_string();
        if let Some(api_key) = api_key {
            this_api_key = api_key;
        }

        NewsApiFetcher {
            api_key: this_api_key,
            country: country.to_string(),
            topic: topic,
        }
    }
}

#[async_trait]
impl NewsFetcher for NewsApiFetcher {
    async fn fetch_news(&mut self) -> Result<Option<(String, String, Option<String>, Option<String>)>, Box<dyn Error>> {
        let mut request_builder = reqwest::Client::new()
            .get("https://newsapi.org/v2/top-headlines")
            .query(&[("country", &self.country), ("apiKey", &self.api_key)])
            .header("User-Agent", "Cool guy");

        if let Some(topic_string) = &self.topic {
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

        let image_url = match first_article["urlToImage"].as_str() {
            None => None,
            Some(image_url) => Some(image_url.to_string()),
        };

        let source = match first_article["source"].as_object() {
            None => "No Source",
            Some(object) => object["name"].as_str().unwrap_or("No Source"),
        };

        let article_link = match first_article["url"].as_str() {
            None => None,
            Some(url) => Some(url.to_string()),
        };

        Ok(Some((title.to_string(), source.to_string(), image_url, article_link)))
    }
}
