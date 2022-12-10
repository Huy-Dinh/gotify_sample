use std::str::FromStr;
use std::sync::Mutex;

use log::debug;
use monitor_grpc_service::monitor_server::{Monitor, MonitorServer};
use monitor_grpc_service::{GetMonitorsReply, GetMonitorsRequest};
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

use crate::grpc_server::monitor_grpc_service::MonitorEntry;
use crate::monitor::top_news_monitor::persistence::TopNewsMonitorPersistence;

use self::monitor_grpc_service::{
    CreateMonitorReply, CreateMonitorRequest, DeleteMonitorReply, DeleteMonitorRequest, MonitorType,
};

pub mod monitor_grpc_service {
    tonic::include_proto!("monitor_service"); // The string specified here must match the proto package name
}

pub struct GrpcMonitorServer {
    monitor_persistence: Arc<Mutex<TopNewsMonitorPersistence>>,
}

impl GrpcMonitorServer {
    pub fn new(persistence: Arc<Mutex<TopNewsMonitorPersistence>>) -> GrpcMonitorServer {
        GrpcMonitorServer {
            monitor_persistence: persistence,
        }
    }
}

#[tonic::async_trait]
impl Monitor for GrpcMonitorServer {
    async fn get_monitors(
        &self,
        request: Request<GetMonitorsRequest>,
    ) -> Result<Response<GetMonitorsReply>, Status> {
        debug!("Got a request: {:?}", request);

        let monitors: Vec<MonitorEntry> = self
            .monitor_persistence
            .lock()
            .unwrap()
            .get_configurations()
            .iter()
            .map(MonitorEntry::from)
            .collect();

        let reply = monitor_grpc_service::GetMonitorsReply { monitors };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }

    async fn delete_monitor(
        &self,
        request: Request<DeleteMonitorRequest>,
    ) -> Result<Response<DeleteMonitorReply>, Status> {
        debug!("Got a delete request: {:?}", request);

        let id_to_delete = uuid::Uuid::from_str(&request.get_ref().id).map_err(|e| {
            Status::invalid_argument(format!("Ill-formed id, must be a uuid. Error: {:?}", e))
        })?;

        match self
            .monitor_persistence
            .lock()
            .unwrap()
            .remove_configuration(&id_to_delete)
        {
            Ok(_) => Ok(Response::new(DeleteMonitorReply {})),
            Err(_) => Err(Status::invalid_argument("No monitor with this Id")),
        }
    }

    async fn create_monitor(
        &self,
        request: Request<CreateMonitorRequest>,
    ) -> Result<Response<CreateMonitorReply>, Status> {
        use crate::monitor::top_news_monitor::config;

        let monitor_config = match request.get_ref().monitor_configuration.as_ref() {
            None => {
                return Err(Status::invalid_argument("Empty request"));
            }
            Some(config) => config,
        };

        if monitor_config.monitor_type == MonitorType::NewsApi as i32
            && monitor_config.news_api_configuration.is_none()
        {
            return Err(Status::invalid_argument("Empty News API configuration"));
        }

        if monitor_config.monitor_type == MonitorType::WebScraper as i32
            && monitor_config.scraper_configuration.is_none()
        {
            return Err(Status::invalid_argument("Empty scraper configuration"));
        }

        let new_monitor_config = config::TopNewsMonitorDatabaseEntry::from(monitor_config);

        let new_monitor_id = new_monitor_config.id.to_string();

        match self
            .monitor_persistence
            .lock()
            .unwrap()
            .add_configuration(new_monitor_config)
        {
            Err(err) => {
                return Err(Status::internal(
                    format! {"Error adding new monitor; {:?}", err},
                ));
            }
            Ok(()) => return Ok(Response::new(CreateMonitorReply { id: new_monitor_id })),
        }
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
