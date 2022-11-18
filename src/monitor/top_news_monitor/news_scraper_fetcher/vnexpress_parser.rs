use super::ParsedNewsDetails;

const IMAGE_SELECTOR: &str = "div.wrapper-topstory-folder>article>div.thumb-art>a>picture>img";
const TITLE_SELECTOR: &str = "div.wrapper-topstory-folder>article>h3>a";

pub fn parse_vnexpress(document: scraper::Html) -> Option<ParsedNewsDetails> {
    let image_selector = scraper::Selector::parse(IMAGE_SELECTOR).unwrap();
    let title_selector = scraper::Selector::parse(TITLE_SELECTOR).unwrap();

    let image = document
        .select(&image_selector)
        .next()
        .and_then(|x| x.value().attr("src"));

    let title_element = document.select(&title_selector).next();

    let title = title_element.map(|x| x.inner_html().trim().to_string());
    let article_url = title_element.and_then(|x| x.value().attr("href"));

    let title = match title {
        None => return None,
        Some(title_value) => title_value,
    };

    let image_url: Option<String> = image.map(|image_url| image_url.to_string());

    Some(ParsedNewsDetails {
        article_url: article_url.map(|url| url.to_string()),
        title,
        image_url,
    })
}
