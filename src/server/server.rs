use std::str::FromStr;

use crate::server::error::ServerError;
use tokio::{io::AsyncWriteExt, net::*};

use super::config::ServerConfig;

pub(crate) struct Server {
    listener: TcpListener,
    stream: TcpStream
}

impl Server {
    pub async fn start(config: ServerConfig) -> Result<Server, ServerError> {
        match TcpListener::bind((String::from_str("0.0.0.0:").unwrap() + &config.port.to_string()).as_str()).await {
            Ok(listener) => {
                let (stream, _addr) = listener.accept().await.unwrap();
                Ok(Server { listener, stream })
            },

            Err(err) => {
                println!("Server Error: [{}]", err);
                Err(ServerError::BindFailed)
            }
        }
    }

    pub fn publish(&mut self) {
        let _ = self.stream.write(&[0; 128]);
    }
}