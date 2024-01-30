
use tokio::net::*;
use thiserror::*;
pub struct Client {
    client: TcpStream
}

#[derive(Error, Debug)]
enum ConnectError { 
    #[error("Connection Failed!")]
    ConnectFailed 
}

impl Client {
    pub async fn connect(addr: &str) -> Result<Client, ConnectError> {
        let result = match TcpStream::connect(addr).await {
            Ok(client) => {
                Ok(Client{ client })
            },
            Err(_err) => {
                Err(ConnectError::ConnectFailed)
            },
        };

        result
    } 


}