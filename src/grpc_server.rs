use std::sync::Mutex;

use log::debug;
use monitor_grpc_service::monitor_server::{Monitor, MonitorServer};
use monitor_grpc_service::{GetMonitorsReply, GetMonitorsRequest};
use tokio::sync::mpsc::Sender;
use tonic::{transport::Server, Request, Response, Status};

use crate::helper::create_monitor;
use crate::monitor::top_news_monitor::persistence::TopNewsMonitorPersistence;
use crate::monitor::MonitorNotification;

use crate::monitor::top_news_monitor::TopNewsMonitor;

use self::monitor_grpc_service::{
    CreateMonitorReply, CreateMonitorRequest, DeleteMonitorReply, DeleteMonitorRequest,
    MonitorConfiguration, MonitorType,
};

pub mod monitor_grpc_service {
    tonic::include_proto!("monitor_service"); // The string specified here must match the proto package name
}

struct MonitorsContainer {
    persistence: TopNewsMonitorPersistence,
    running_monitors: Vec<TopNewsMonitor>,
}

pub struct GrpcMonitorServer {
    monitors_container: Mutex<MonitorsContainer>,
    sender: Mutex<Sender<MonitorNotification>>,
}

impl GrpcMonitorServer {
    pub fn new(
        persistence: TopNewsMonitorPersistence,
        running_monitors: Vec<TopNewsMonitor>,
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

        let monitor_configurations: Vec<MonitorConfiguration> = self
            .monitors_container
            .lock()
            .unwrap()
            .persistence
            .get_configurations()
            .iter()
            .map(MonitorConfiguration::from)
            .collect();

        let reply = monitor_grpc_service::GetMonitorsReply {
            monitor_configurations,
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }

    async fn delete_monitor(
        &self,
        request: Request<DeleteMonitorRequest>,
    ) -> Result<Response<DeleteMonitorReply>, Status> {
        debug!("Got a delete request: {:?}", request);

        let index_to_delete = request.get_ref().index as usize;

        let mut monitors_container = self.monitors_container.lock().unwrap();

        if index_to_delete >= monitors_container.running_monitors.len() {
            Err(Status::invalid_argument("Index out of range"))
        } else {
            monitors_container
                .persistence
                .remove_configuration(index_to_delete)
                .unwrap();
            monitors_container.running_monitors.remove(index_to_delete);

            Ok(Response::new(DeleteMonitorReply {}))
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

        let new_monitor_config = config::MonitorConfiguration::from(monitor_config);

        let mut monitors_container = self.monitors_container.lock().unwrap();

        monitors_container.running_monitors.push(create_monitor(
            self.sender.lock().unwrap().clone(),
            &new_monitor_config,
        ));
        match monitors_container
            .persistence
            .add_configuration(new_monitor_config)
        {
            Err(err) => {
                return Err(Status::internal(
                    format! {"Error adding new monitor; {:?}", err},
                ));
            }
            Ok(()) => return Ok(Response::new(CreateMonitorReply {})),
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
