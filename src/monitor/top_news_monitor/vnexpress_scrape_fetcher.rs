use std::error::Error;

use super::{NewsFetcher, NewsInfo, ResponseParsingFailed};
use async_trait::async_trait;
use log::info;
use scraper::Selector;

const IMAGE_SELECTOR: &str = "div.wrapper-topstory-folder>article>div.thumb-art>a>picture>img";
const TITLE_SELECTOR: &str = "div.wrapper-topstory-folder>article>h3>a";
const VNEXPRESS_SOURCE_NAME: &str = "VnExpress";
const VNEXPRESS_URL: &str = "https://vnexpress.net/";

pub struct VnExpressScrapeFetcher {
    last_title: String,
    image_selector: Selector,
    title_selector: Selector,
    url_to_scrape: String,
    client: reqwest::Client,
}

impl VnExpressScrapeFetcher {
    pub fn new() -> VnExpressScrapeFetcher {
        let mut url_to_scrape = VNEXPRESS_URL.to_string();

        VnExpressScrapeFetcher {
            last_title: "".to_string(),
            image_selector: scraper::Selector::parse(IMAGE_SELECTOR).unwrap(),
            title_selector: scraper::Selector::parse(TITLE_SELECTOR).unwrap(),
            url_to_scrape: url_to_scrape,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NewsFetcher for VnExpressScrapeFetcher {
    async fn fetch_news(&mut self) -> Result<Option<NewsInfo>, Box<dyn Error>> {
        info!("Fetching {}", &self.url_to_scrape);

        let response = self.client.get(&self.url_to_scrape).send().await?;
        info!("Done fetching {}", &self.url_to_scrape);

        let response = response.text().await?;

        let document = scraper::Html::parse_document(&response);

        let image = document
            .select(&self.image_selector)
            .next()
            .and_then(|x| x.value().attr("src"));

        let title_element = document.select(&self.title_selector).next();
        let title = title_element.map(|x| x.inner_html().trim().to_string());
        let article_url = title_element.and_then(|x| x.value().attr("href"));

        let title = match title {
            None => return Err(ResponseParsingFailed.into()),
            Some(title_value) => title_value,
        };

        if title.eq(&self.last_title) {
            return Ok(None);
        }

        self.last_title = title.clone();

        let image_url: Option<String> = image.map(|image_url| image_url.to_string());

        let news_info = NewsInfo {
            title: title,
            source: VNEXPRESS_SOURCE_NAME.to_string(),
            image_url: image_url,
            article_url: article_url.map(|url| url.to_string()),
        };

        Ok(Some(news_info))
    }
}
