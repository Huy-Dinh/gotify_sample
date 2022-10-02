use std::error::Error;

use super::{NewsFetcher, NewsInfo, ResponseParsingFailed};
use async_trait::async_trait;
use log::info;
use scraper::Selector;

const IMAGE_SELECTOR: &str = "div.shnews_box>div.hl-img>a>img";
const TITLE_SELECTOR: &str = "div.shnews_box>div.shnews_total>h3>a";
const SOHA_SOURCE_NAME: &str = "Soha";
const SOHA_URL: &str = "https://soha.vn";

pub struct SohaScrapeFetcher {
    last_title: String,
    image_selector: Selector,
    title_selector: Selector,
    url_to_scrape: String,
    client: reqwest::Client,
}

impl SohaScrapeFetcher {
    pub fn new(path_to_scrape: &str) -> SohaScrapeFetcher {
        let mut url_to_scrape = SOHA_URL.to_string();
        url_to_scrape.push_str("/");
        url_to_scrape.push_str(path_to_scrape);

        SohaScrapeFetcher {
            last_title: "".to_string(),
            image_selector: scraper::Selector::parse(IMAGE_SELECTOR).unwrap(),
            title_selector: scraper::Selector::parse(TITLE_SELECTOR).unwrap(),
            url_to_scrape: url_to_scrape,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NewsFetcher for SohaScrapeFetcher {
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

        let article_url = article_url.map(|article_url| {
            let mut full_link = SOHA_URL.to_string();
            full_link.push_str(article_url);
            full_link
        });

        let news_info = NewsInfo {
            title: title,
            source: SOHA_SOURCE_NAME.to_string(),
            image_url: image_url,
            article_url: article_url,
        };

        Ok(Some(news_info))
    }
}
