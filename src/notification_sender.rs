use reqwest::StatusCode;
use serde_json::{self, json};
use core::fmt;
use std::error::Error;
use url::Url;

#[derive(Debug, Clone)]
struct RequestFailed {
    url: String,
    message: String
}

impl fmt::Display for RequestFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Request failed. Url: {}, message: {}", self.url, self.message)
    }
}

impl Error for RequestFailed {}

pub async fn send_notification(
    base_url: &Url,
    app_token: &str,
    title: &str,
    message: &str,
    priority: u32,
) -> Result<(), Box<dyn Error>> {

    let notification_json = json!({
            "message": message,
            "title": title,
            "priority": priority
    });

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
        },
        _ => {
            let response_text = response.text().await?;
            return Err(RequestFailed {
                message: response_text,
                url: url.to_string()
            }.into());
        }
    }
}
