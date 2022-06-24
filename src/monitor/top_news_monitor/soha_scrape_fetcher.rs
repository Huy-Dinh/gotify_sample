use std::error::Error;

use super::{NewsFetcher, ResponseParsingFailed};
use async_trait::async_trait;
use scraper::Selector;

const IMAGE_SELECTOR: &str = "div.shnews_box>div.hl-img>a>img";
const TITLE_SELECTOR: &str = "div.shnews_box>div.shnews_total>h3>a";
const SOHA_SOURCE_NAME: &str = "Soha";
const SOHA_URL: &str = "https://soha.vn";

pub struct SohaScrapeFetcher {
    last_title: String,
    image_selector: Selector,
    title_selector: Selector,
    url_to_scrape: String
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
            url_to_scrape: url_to_scrape
        }
    }
}

#[async_trait]
impl NewsFetcher for SohaScrapeFetcher {
    async fn fetch_news(
        &mut self,
    ) -> Result<Option<(String, String, Option<String>, Option<String>)>, Box<dyn Error>> {
        let response = reqwest::get(&self.url_to_scrape)
            .await?
            .text()
            .await?;

        let document = scraper::Html::parse_document(&response);

        let image = document
            .select(&self.image_selector)
            .next()
            .and_then(|x| x.value().attr("src"));

        let title_element = document.select(&self.title_selector).next();
        let title = title_element.map(|x| x.inner_html().trim().to_string());
        let article_link = title_element.and_then(|x| x.value().attr("href"));

        let title = match title {
            None => return Err(ResponseParsingFailed.into()),
            Some(title_value) => title_value,
        };

        if title.eq(&self.last_title) {
            return Ok(None);
        }

        self.last_title = title.clone();

        let image_url: Option<String> = image.map(|image_url| image_url.to_string());

        let article_link = article_link.map(|article_link| {
            let mut full_link = SOHA_URL.to_string();
            full_link.push_str(article_link);
            full_link
        });

        Ok(Some((title, SOHA_SOURCE_NAME.to_string(), image_url, article_link)))
    }
}
