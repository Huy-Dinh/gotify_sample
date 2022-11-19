use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

use log::debug;
use monitor_grpc_service::monitor_server::{Monitor, MonitorServer};
use monitor_grpc_service::{GetMonitorsReply, GetMonitorsRequest};
use tokio::sync::mpsc::Sender;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use crate::grpc_server::monitor_grpc_service::MonitorEntry;
use crate::helper::create_monitor;
use crate::monitor::top_news_monitor::persistence::TopNewsMonitorPersistence;
use crate::monitor::MonitorNotification;

use crate::monitor::top_news_monitor::TopNewsMonitor;

use self::monitor_grpc_service::{
    CreateMonitorReply, CreateMonitorRequest, DeleteMonitorReply, DeleteMonitorRequest, MonitorType,
};

pub mod monitor_grpc_service {
    tonic::include_proto!("monitor_service"); // The string specified here must match the proto package name
}

struct MonitorsContainer {
    persistence: TopNewsMonitorPersistence,
    running_monitors: HashMap<Uuid, TopNewsMonitor>,
}

pub struct GrpcMonitorServer {
    monitors_container: Mutex<MonitorsContainer>,
    sender: Mutex<Sender<MonitorNotification>>,
}

impl GrpcMonitorServer {
    pub fn new(
        persistence: TopNewsMonitorPersistence,
        running_monitors: HashMap<Uuid, TopNewsMonitor>,
        sender: Sender<MonitorNotification>,
    ) -> GrpcMonitorServer {
        GrpcMonitorServer {
            monitors_container: Mutex::new(MonitorsContainer {
                persistence,
                running_monitors,
            }),
            sender: Mutex::new(sender),
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
            .monitors_container
            .lock()
            .unwrap()
            .persistence
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

        let mut monitors_container = self.monitors_container.lock().unwrap();

        if monitors_container
            .running_monitors
            .contains_key(&id_to_delete)
        {
            monitors_container.running_monitors.remove(&id_to_delete);
            monitors_container
                .persistence
                .remove_configuration(&id_to_delete)
                .unwrap();
            Ok(Response::new(DeleteMonitorReply {}))
        } else {
            Err(Status::invalid_argument(format!(
                "No monitor with ID {id_to_delete} found"
            )))
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

        let mut monitors_container = self.monitors_container.lock().unwrap();

        monitors_container.running_monitors.insert(
            new_monitor_config.id.clone(),
            create_monitor(self.sender.lock().unwrap().clone(), &new_monitor_config),
        );

        let new_monitor_id = new_monitor_config.id.to_string();

        match monitors_container
            .persistence
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
