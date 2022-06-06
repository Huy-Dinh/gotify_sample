use url::Url;

mod notification_sender;

const BASE_URL_STRING: &'static str = "https://gotify.van-ngo.com";

// tokio let's us use "async" on our main function
#[tokio::main]
async fn main() {
    let base_url = Url::parse(BASE_URL_STRING).expect("Failed to parse the base url");

    match notification_sender::send_notification(
        &base_url,
        "A7opbHJXd4qnc7Z",
        "Hello",
        "From the otter slide",
        10,
    )
    .await
    {
        Err(e) => {
            println!("{}", e);
        }
        Ok(()) => {
            println!("All good");
        }
    }
}
