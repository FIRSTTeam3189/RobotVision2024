use std::net::SocketAddr;

use crate::{server::error::ServerError, NetworkConfig};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::*};

pub(crate) struct Server {
    listener: TcpListener,
    stream: TcpStream
}

impl Server {
    pub async fn start(config: NetworkConfig) -> Result<Server, ServerError> {
        let ip = SocketAddr::from(([0,0,0,0], config.server_port as u16));
        match TcpListener::bind(&ip).await {
            Ok(listener) => {
                println!("Listener Started!");
                let (stream, addr) = listener.accept().await.unwrap();
                println!("{}", addr.to_string());
                println!("Found Connection to Server!");
                Ok(Server { listener, stream })
            },

            Err(err) => {
                println!("Server Error: [{}]", err);
                Err(ServerError::BindFailed)
            }
        }
    }

    pub fn publish<'a>(&mut self, msg: &'a [u8]) {
        let _ = self.stream.write(msg);
    }

    pub async fn read(&mut self) -> usize {
        let mut x = vec![];
        self.stream.read(&mut x).await.unwrap()
    }
}