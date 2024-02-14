use std::sync::Arc;
use crossbeam_channel::Receiver;
use tokio::runtime::Handle;
use crate::{nt::client, process::VisionData, server::{self, server::Server}, NetworkConfig};

#[cfg(feature = "nt")]
use crate::nt::client::*;

#[cfg(not(feature = "nt"))]
use crate::{server, NetworkConfig};

pub(crate) struct Network {
    config: NetworkConfig,
    data_rx: Arc<Receiver<VisionData>>
}

impl Network {
    pub fn new(config: NetworkConfig, data_rx: Receiver<VisionData>) -> Self {
       let data_rx = Arc::new(data_rx);
        Network{config, data_rx}
    }

    #[cfg(feature = "nt")]
    pub async fn update(&mut self, runtime: &Handle) {

        let config = self.config.clone();
        let mut client = client::NT::new(config).await;
        let data_rx = self.data_rx.clone();
        runtime.spawn(async move {
            loop {
                match data_rx.recv() {
                    Ok(data) => {
                        client.publish(data).await;
                    },
                    Err(_) => {}
                }
            }
        });
    }

    #[cfg(not(feature = "nt"))] 
    pub fn update(&mut self, runtime: &Handle) { 
        let config = self.config.clone();
        let mut server = Server::start(self.config);
        let data_rx = self.data_rx.clone();
        runtime.spawn(async move {
            loop {
                match data_rx.recv() {
                    Ok(data) => {
                        server.publish(data.into_bytes());
                    },
                    Err(_) => {}
                }
            }
        });

    }
}