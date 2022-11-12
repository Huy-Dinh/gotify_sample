use monitor_service::monitor_client::MonitorClient;
use monitor_service::{GetMonitorsRequest, DeleteMonitorRequest};

pub mod monitor_service {
    tonic::include_proto!("monitor_service");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MonitorClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(GetMonitorsRequest {});

    let response = client.get_monitors(request).await?;

    println!("RESPONSE={:?}", response);

    let request = tonic::Request::new(DeleteMonitorRequest {index: 0});

    let response = client.delete_monitor(request).await?;

    println!("RESPONSE={:?}", response);

    let request = tonic::Request::new(DeleteMonitorRequest {index: 100});

    let response = client.delete_monitor(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}