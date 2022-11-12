use std::sync::Mutex;

use log::debug;
use monitor_grpc_service::monitor_server::{Monitor, MonitorServer};
use monitor_grpc_service::{GetMonitorsReply, GetMonitorsRequest};
use tonic::{transport::Server, Request, Response, Status};

use crate::monitor::top_news_monitor::persistence::TopNewsMonitorPersistence;

use crate::monitor::top_news_monitor::TopNewsMonitor;

use self::monitor_grpc_service::{DeleteMonitorReply, DeleteMonitorRequest, MonitorConfiguration};

pub mod monitor_grpc_service {
    tonic::include_proto!("monitor_service"); // The string specified here must match the proto package name
}

pub struct GrpcMonitorServer {
    persistence: Mutex<TopNewsMonitorPersistence>,
    running_monitors: Mutex<Vec<TopNewsMonitor>>,
}

impl GrpcMonitorServer {
    pub fn new(
        persistence: TopNewsMonitorPersistence,
        running_monitors: Vec<TopNewsMonitor>,
    ) -> GrpcMonitorServer {
        GrpcMonitorServer {
            persistence: Mutex::new(persistence),
            running_monitors: Mutex::new(running_monitors),
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
            .persistence
            .lock()
            .unwrap()
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

        if index_to_delete >= self.running_monitors.lock().unwrap().len() {
            Err(Status::invalid_argument("Index out of range"))
        } else {
            self.persistence
                .lock()
                .unwrap()
                .remove_configuration(index_to_delete)
                .unwrap();
            self.running_monitors
                .lock()
                .unwrap()
                .remove(index_to_delete);

            Ok(Response::new(DeleteMonitorReply {}))
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
