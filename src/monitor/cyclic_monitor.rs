use super::MonitorNotification;
use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime};

pub struct CyclicMonitor;

const APP_TOKEN: &'static str = "A7opbHJXd4qnc7Z";

impl CyclicMonitor {
    pub fn start(&self, sender: Sender<MonitorNotification>) {
        let running_fn = async move {
            loop {
                let notification = MonitorNotification {
                    app_token: APP_TOKEN,
                    title: "Time report my man".to_string(),
                    message: format!("{:?}", SystemTime::now())
                };
    
                match sender.send(notification) {
                    Err(e) => {
                        println!("Error sending from cyclic monitor {:?}", e);
                    }
                    Ok(_) => {}
                }

                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        };

        tokio::spawn(running_fn);
    }
}
