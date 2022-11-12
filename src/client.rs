use monitor_service::monitor_client::MonitorClient;
use monitor_service::GetMonitorsRequest;

pub mod monitor_service {
    tonic::include_proto!("monitor_service");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MonitorClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(GetMonitorsRequest {});

    let response = client.get_monitors(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}