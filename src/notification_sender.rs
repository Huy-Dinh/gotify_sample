use core::fmt;
use reqwest::StatusCode;
use serde_json::{self, json};
use std::error::Error;
use url::Url;

#[derive(Debug, Clone)]
struct RequestFailed {
    url: String,
    message: String,
}

impl fmt::Display for RequestFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Request failed. Url: {}, message: {}",
            self.url, self.message
        )
    }
}

impl Error for RequestFailed {}

pub async fn send_notification(
    base_url: &Url,
    app_token: &str,
    title: &str,
    message: &str,
    image_url: &Option<String>,
    article_link: &Option<String>,
    priority: u32,
) -> Result<(), Box<dyn Error>> {
    let mut notification_json = json!({
            "message": message,
            "title": title,
            "priority": priority
    });

    let mut client_notification = json!({});

    if let Some(article_link) = article_link {
        client_notification["click"] = json!({ "url": article_link });
    }

    if let Some(image_url) = image_url {
        client_notification["bigImageUrl"] = json!(image_url);
    }

    if client_notification.is_object() {
        notification_json["extras"] = json!({ "client::notification": client_notification });
    }

    let value = base_url.join("message")?;
    let url = value.as_str();

    let response = reqwest::Client::new()
        .post(url)
        .header("X-Gotify-Key", app_token)
        .json(&notification_json)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            return Ok(());
        }
        _ => {
            let response_text = response.text().await?;
            return Err(RequestFailed {
                message: response_text,
                url: url.to_string(),
            }
            .into());
        }
    }
}
