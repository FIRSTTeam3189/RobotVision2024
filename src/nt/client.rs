use std::borrow::BorrowMut;
use serde::{Deserialize, Serialize};
use tokio_util::{bytes::{Buf, BytesMut}, codec::*};
use tokio::net::*;
use thiserror::*;
pub(crate) struct Client {
    client: TcpStream
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Message {
    key: String,
    value: i8
}

#[derive(Error, Debug)]
pub enum ConnectError { 
    #[error("Connection Failed!")]
    ConnectFailed 
}

impl Client {
    pub async fn connect(addr: &str) -> Result<Client, ConnectError> {
        let result = match TcpStream::connect(addr).await {
            Ok(client) => {
                println!("\t\tNetwork Tables Connected!!!!!!!!!!");
                Ok(Client{ client })
            },
            Err(_err) => {
                println!("\t\t Network Tables Failed!!!!!!!!");
                Err(ConnectError::ConnectFailed)
            },
        };

        result
    } 

    pub fn send_message(&mut self) {
        let message = Message {key: "Test".to_string(), value: 10 };
        let message = serde_json::to_string(&message).unwrap();

        let packet = Packet(serde_json::Value::String(message));
        let frame = Framed::new(self.client.borrow_mut(), packet);

        frame.write_buffer();
    }
}

struct Packet (serde_json::Value);

impl Decoder for Packet {
    type Item = serde_json::Value;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();
        let data = src[0..len].to_vec();
        match String::from_utf8(data) {
            Ok(string) => match serde_json::from_str(&string) {
                Ok(v) => {
                    src.advance(len);
                    Ok(Some(v))
                }
                Err(err) => {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("{err}"),
                    ))
                }
            },
            Err(utf8_error) => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    utf8_error.utf8_error(),
                ))
            },
        }
    }
}

impl Encoder<String> for Packet {
    type Error = std::io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let len = item.len();

        let len_slice = u32::to_le_bytes(item.len() as u32);
        dst.reserve(4 + len);
        dst.extend_from_slice(&len_slice);
        dst.extend_from_slice(item.as_bytes());
        Ok(())
    }
}
