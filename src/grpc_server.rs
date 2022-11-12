use monitor_grpc_service::monitor_server::{Monitor, MonitorServer};
use monitor_grpc_service::{GetMonitorsReply, GetMonitorsRequest};
use tonic::{transport::Server, Request, Response, Status};

use crate::monitor::top_news_monitor::persistence::TopNewsMonitorPersistence;

use crate::monitor::top_news_monitor::config;

use self::monitor_grpc_service::{MonitorConfiguration, NewsApiConfiguration, ScraperApiConfiguration, MonitorType, ParserType};

pub mod monitor_grpc_service {
    tonic::include_proto!("monitor_service"); // The string specified here must match the proto package name
}

pub struct GrpcMonitorServer {
    persistence: TopNewsMonitorPersistence,
}

impl GrpcMonitorServer {
    pub fn new(persistence: TopNewsMonitorPersistence) -> GrpcMonitorServer {
        GrpcMonitorServer { persistence }
    }
}

impl From<&config::ParserType> for ParserType {
    fn from(parser_type: &config::ParserType) -> Self {
        match parser_type {
            config::ParserType::Soha => ParserType::Soha,
            config::ParserType::VnExpress => ParserType::Vnexpress
        }
    }
}

impl From<&config::MonitorConfiguration> for MonitorConfiguration {
    fn from(monitor_config: &config::MonitorConfiguration) -> Self {

        let mut news_api_configuration: Option<NewsApiConfiguration> = None;
        let mut scraper_api_configuration: Option<ScraperApiConfiguration> = None;
        let monitor_type: i32;

        match &monitor_config.monitor_type {
            config::MonitorType::ApiMonitor { api_key, country, topic } => {
                monitor_type = MonitorType::NewsApi as i32;
                news_api_configuration = Some(NewsApiConfiguration {
                    api_key: api_key.clone(),
                    country: country.clone(),
                    topic: topic.clone()
                });
            },
            config::MonitorType::ScraperMonitor { url, name, parser_type } => {
                monitor_type = MonitorType::WebScraper as i32;

                scraper_api_configuration = Some(ScraperApiConfiguration{
                    url: url.clone(),
                    name: name.clone(),
                    parser_type: ParserType::from(parser_type) as i32
                });
            }
        }

        MonitorConfiguration { 
            interval_in_seconds: monitor_config.interval.as_secs(), 
            monitor_type,
            news_api_configuration,
            scraper_api_configuration 
        }
    }
}

#[tonic::async_trait]
impl Monitor for GrpcMonitorServer {
    async fn get_monitors(
        &self,
        request: Request<GetMonitorsRequest>,
    ) -> Result<Response<GetMonitorsReply>, Status> {
        println!("Got a request: {:?}", request);

        let monitor_configurations: Vec<MonitorConfiguration> = self.persistence.get_configurations().iter().map(MonitorConfiguration::from).collect();

        let reply = monitor_grpc_service::GetMonitorsReply {
            monitor_configurations
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

pub async fn start_server(
    port_number: u32,
    server: GrpcMonitorServer,
) -> tokio::task::JoinHandle<()> {
    let addr = format!("[::1]:{port_number}").parse().unwrap();

    let start_fn = async move {
        Server::builder()
            .add_service(MonitorServer::new(server))
            .serve(addr)
            .await
            .unwrap();
    };

    tokio::spawn(start_fn)
}
