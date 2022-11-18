use monitor_service::monitor_client::MonitorClient;
use monitor_service::{
    DeleteMonitorRequest, GetMonitorsRequest, MonitorConfiguration, ScraperApiConfiguration,
};

use crate::monitor_service::{CreateMonitorRequest, NewsApiConfiguration};

pub mod monitor_service {
    tonic::include_proto!("monitor_service");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MonitorClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(GetMonitorsRequest {});

    let response = client.get_monitors(request).await?;

    println!("RESPONSE={:?}", response);

    let request = tonic::Request::new(DeleteMonitorRequest { index: 0 });

    let response = client.delete_monitor(request).await?;

    println!("RESPONSE={:?}", response);

    let request = tonic::Request::new(CreateMonitorRequest {
        monitor_configuration: Some(MonitorConfiguration {
            interval_in_seconds: 1800,
            monitor_type: 1,
            scraper_configuration: Some(ScraperApiConfiguration {
                name: String::from("New shiet"),
                url: String::from("https://soha.vn/giai-tri.htm"),
                parser_type: 0,
            }),
            news_api_configuration: None,
        }),
    });

    let response = client.create_monitor(request).await?;

    println!("RESPONSE={:?}", response);

    let request = tonic::Request::new(CreateMonitorRequest {
        monitor_configuration: Some(MonitorConfiguration {
            interval_in_seconds: 1800,
            monitor_type: 0,
            scraper_configuration: None,
            news_api_configuration: Some(NewsApiConfiguration {
                api_key: None,
                topic: None,
                country: String::from("de"),
            }),
        }),
    });

    let response = client.create_monitor(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
