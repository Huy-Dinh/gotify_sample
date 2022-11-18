use super::{NewsFetcher, NewsInfo};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::info;
use std::{sync::Mutex, time::Duration};

pub mod soha_parser;
pub mod vnexpress_parser;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
pub struct ParsedNewsDetails {
    title: String,
    image_url: Option<String>,
    article_url: Option<String>,
}

pub struct NewsScraperFetcher {
    last_title: Mutex<String>,
    url_to_scrape: String,
    source_name: String,
    client: reqwest::Client,
    parse: fn(scraper::Html) -> Option<ParsedNewsDetails>,
}

impl NewsScraperFetcher {
    pub fn new(
        source_name: &str,
        url_to_scrape: &str,
        parse: fn(scraper::Html) -> Option<ParsedNewsDetails>,
    ) -> NewsScraperFetcher {
        NewsScraperFetcher {
            last_title: Mutex::new(String::new()),
            url_to_scrape: url_to_scrape.to_string(),
            source_name: source_name.to_string(),
            client: reqwest::Client::new(),
            parse,
        }
    }
}

#[async_trait]
impl NewsFetcher for NewsScraperFetcher {
    async fn fetch_news(&self) -> Result<Option<NewsInfo>> {
        info!("Fetching {}", &self.url_to_scrape);

        let response = self
            .client
            .get(&self.url_to_scrape)
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await?;
        info!("Done fetching {}", &self.url_to_scrape);

        let response = response.text().await?;

        let document = scraper::Html::parse_document(&response);

        let parsed_news_details = (self.parse)(document);

        let parsed_news_details = match parsed_news_details {
            None => return Ok(None),
            Some(news_details) => news_details,
        };

        {
            let mut last_title = self
                .last_title
                .lock()
                .map_err(|_| anyhow!("Failed to lock last title mutex"))?;

            if parsed_news_details.title.eq(&*last_title) {
                return Ok(None);
            }

            *last_title = parsed_news_details.title.clone();
        }

        let news_info = NewsInfo {
            title: parsed_news_details.title,
            source: self.source_name.to_owned(),
            image_url: parsed_news_details.image_url,
            article_url: parsed_news_details.article_url,
        };

        Ok(Some(news_info))
    }
}
