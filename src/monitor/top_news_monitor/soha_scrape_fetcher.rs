use std::error::Error;

use super::{NewsFetcher, ResponseParsingFailed};
use async_trait::async_trait;

pub struct SohaScrapeFetcher;

#[async_trait]
impl NewsFetcher for SohaScrapeFetcher {
    async fn fetch_news(
        &self,
    ) -> Result<Option<(String, String, Option<String>, Option<String>)>, Box<dyn Error>> {
        let response = reqwest::get("https://soha.vn/quoc-te.htm")
            .await?
            .text()
            .await?;

        let document = scraper::Html::parse_document(&response);

        let image_selector = scraper::Selector::parse("div.shnews_box>div.hl-img>a>img").unwrap();
        let image = document
            .select(&image_selector)
            .next()
            .and_then(|x| x.value().attr("src"));

        let title_selector =
            scraper::Selector::parse("div.shnews_box>div.shnews_total>h3>a").unwrap();

        let title_element = document.select(&title_selector).next();

        let title = title_element.map(|x| x.inner_html().trim().to_string());
        let article_link = title_element.and_then(|x| x.value().attr("href"));

        let title = match title {
            None => return Err(ResponseParsingFailed.into()),
            Some(title_value) => title_value,
        };

        let image_url: Option<String> = match image {
            None => None,
            Some(image) => Some(image.to_string()),
        };

        let article_link = match article_link {
            None => None,
            Some(article_link) => Some({
                let mut full_link = "https://soha.vn".to_string();
                full_link.push_str(article_link);
                full_link
            })
        };

        Ok(Some((title, "Soha".to_string(), image_url, article_link)))
    }
}
