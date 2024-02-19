//! # Interface for VisionData
//!
//! This module contains the `DataInterface`, `DataError`, `SyncSequenceCodec` objects and the `open_serial_port`, `open_tcp_stream` functions.
//!
//! The `DataInterface` is a wrapper around sending and receiving data from any async read/write source for `VisionData` packets.
//! This allows us to use either a TCP, and Serial port (or other stream types implementing `AsyncRead` and `AsyncWrite`) in a generic way and change out the interface to talk to the robot without changing the rest of the code.
//!
//! The `open_serial_port` and `open_tcp_stream` functions are helper functions to open up a serial port or TCP stream with the desired settings and create an appropriate `DataInterface` object.
//!
//! The `SyncSequenceCodec` is a `tokio_util::codec::Decoder` and `tokio_util::codec::Encoder` for the `Framed` object in the `DataInterface` to handle the sync sequence bytes for reading and writing `VisionData` packets.
//!
//! The `DataError` is the error type for any data interface errors, including Serial/TCP/UDP/IO/etc... errors.
//!
//! The `DEFAULT_SYNC_BYTES` is the synchronization bytes to append to the beginning of every `VisionData` packet.
//!
//! # Example
//! ```no_run
//! use std::path::Path;
//! use interface::{open_serial_port, DataError};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), DataError> {
//!    let port = Path::new("/dev/ttyUSB0");
//!    let mut data = open_serial_port(port).await?;
//!
//!    // Write "Hello, World!" to the serial port
//!    data.write_bytes(b"Hello, World!").await?;
//!
//!    // Write some `VisionData` to the serial port
//!    data.write_vision_data(VisionData::new(
//!                            true,
//!                             tag.id() as u64,
//!                             0.0,
//!                             [rotation.0, rotation.1, rotation.2],
//!                             [transform.x, transform.y, transform.z]
//!    )).await?;
//!   // Read a response from the serial port
//!   let response = data.read_frame().await?;
//!   println!("Response: {:?}", response);
//!
//!   Ok(())
//! }
//! ```
use std::{net::SocketAddr, path::Path};
use std::borrow::BorrowMut;
use bondrewd::Bitfields;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, Decoder, Encoder};
use tokio_serial::{DataBits, FlowControl, Parity, SerialPortBuilderExt, StopBits};
use tokio_util::bytes::{Buf, Bytes, BytesMut};

use thiserror::Error;

use futures::{future, SinkExt, StreamExt, TryStreamExt};
use crate::process::VisionData;
use crate::InterfaceConfig;

/// Error type for any data interface errors.
///
/// This also includes Serial/TCP/UDP/IO/etc... errors
#[derive(Error, Debug)]
pub enum DataError {
    /// I/O error
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    /// Serial port error
    #[error("Serial Error: {0}")]
    Serial(#[from] tokio_serial::Error),
    /// Codec error
    #[error("Codec Error: {0}")]
    Codec(#[from] tokio_util::codec::AnyDelimiterCodecError),
    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Server Creation Failed: {0}")]
    ServerCreationFailed(std::io::Error),
    /// No response error
    #[error("No response")]
    NoResponse,
}

/// The synchronization bytes to append to the beginning of every `VisionData` packet.
pub const DEFAULT_SYNC_BYTES: [u8; 4] = [0x1A, 0xCF, 0xFC, 0x1D];

// --- Support stuff to allow us to exchange in a generic way between serial, TCP and UDP ---
/// Supertrait to express Rust type that implements both AsyncRead/Write.
pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T> AsyncReadWrite for T where T: AsyncRead + AsyncWrite + Send + Unpin {}
// --- Support stuff to allow us to exchange in a generic way between serial, TCP and UDP ---

// --- Default serial settings to use when opening the serial port ---
pub const DEFAULT_BAUD_RATE: u32 = 115_200;
pub const DEFAULT_DATA_BITS: DataBits = DataBits::Eight;
pub const DEFAULT_PARITY: Parity = Parity::None;
pub const DEFAULT_STOP_BITS: StopBits = StopBits::One;
pub const DEFAULT_FLOW_CONTROL: FlowControl = FlowControl::None;
// --- Default serial settings to use when opening the serial port ---

// --- Implementation of serial port ---
/// This is a wrapper around the tokio-serial crate to provide a more ergonomic interface for
/// opening, writing, and reading from a serial port.
///
/// This will open up the serial port with the desired default settings then create an appropriate `DataInterface` object.
pub async fn open_serial_port(config: &InterfaceConfig) -> Result<DataInterface, DataError> {
    let port = Path::new(&config.serial_port);
    // Check that the path exists
    if !port.exists() {
        return Err(DataError::NotFound(format!("Serial port not found: {:?}", port)));
    }
    // Open the serial port
    let serial = tokio_serial::new(port.to_string_lossy(), DEFAULT_BAUD_RATE)
        .data_bits(DEFAULT_DATA_BITS)
        .parity(DEFAULT_PARITY)
        .stop_bits(DEFAULT_STOP_BITS)
        .flow_control(DEFAULT_FLOW_CONTROL)
        .open_native_async()?;
    Ok(DataInterface::new(Box::new(serial)))
}
// --- Implementation of serial port ---

// --- Implementation of TCP ---
/// This is a wrapper around the tokio::net::TcpStream to provide a more ergonomic interface for
/// opening, writing, and reading from a TCP stream.
///
/// This will open up the TCP stream with the desired IP and port then create an appropriate `DataInterface` object.
pub async fn _open_tcp_stream<S: AsRef<str>>(ip: S, port: u16) -> Result<DataInterface, DataError> {
    let ip = ip.as_ref();
    match tokio::net::TcpStream::connect(format!("{}:{}", ip, port)).await {
        Ok(stream) => {
            Ok(DataInterface::new(Box::new(stream)))
        },
        Err(_) => {
            Err(DataError::NoResponse)
        },
    }
    
}
// --- Implementation of TCP ---

// --- Implemantation of TCP Server ---
/// This is a wrapper around tokio::net::Listener & tokio::net::Stream to create a listener and start listening for connections
/// and start a stream after a connection has been accepted 
pub async fn start_tcp_server(config: &InterfaceConfig) -> Result<DataInterface, DataError>{
    let ip = SocketAddr::from(([0,0,0,0], config.server_port));
    match TcpListener::bind(&ip).await {
        Ok(listener) => {
            println!("Listener Started!");
            let (stream, addr) = listener.accept().await.unwrap();
            println!("{}", addr.to_string());
            println!("Found Connection to Server!");
            Ok(DataInterface::new(Box::new(stream)))
        }

        Err(err) => {
            Err(DataError::ServerCreationFailed(err))
        }
    }
}
// --- Implemantation of TCP Server ---


// --- Implementation of SyncSequenceCodec ---
/// The data structure object containing the sync sequence bytes
#[derive(Debug, Clone)]
pub struct SyncSequenceCodec {
    /// The sync sequence bytes on read
    sync_sequence_read: Vec<u8>,
    /// The sync sequence bytes on write
    sync_sequence_write: Vec<u8>,
}

impl SyncSequenceCodec {
    /// Creates a new `SyncSequenceCodec` object with the given sync sequence bytes.
    pub fn new(sync_sequence_read: Vec<u8>, sync_sequence_write: Vec<u8>) -> Self {
        Self {
            sync_sequence_read,
            sync_sequence_write,
        }
    }
}

impl Decoder for SyncSequenceCodec {
    type Item = Vec<u8>;
    type Error = DataError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Find the sync sequence in the buffer
        let start = src.windows(self.sync_sequence_read.len()).position(|v| v == self.sync_sequence_read);
        if let Some(start) = start {
            // Get all bytes after the start of the sync sequence, via slice
            let data = &src[start..];
            // Find the next sync sequence in the buffer
            let end = data.windows(self.sync_sequence_read.len()).position(|v| v == self.sync_sequence_read);
            if let Some(end) = end {
                // Remove the sync sequence from the buffer
                let data = &data[..end];
                let data = data.to_vec();

                // Remove the start and data up to next sync from the src buffer
                src.advance(start + end);

                Ok(Some(data))
            } else {
                // No end sync sequence found
                Ok(None)
            }
        } else {
            // No sync sequence found
            Ok(None)
        }
    }
}

impl Encoder<Bytes> for SyncSequenceCodec {
    type Error = DataError;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Append the sync sequence to the beginning of the buffer
        dst.extend_from_slice(&self.sync_sequence_write);
        dst.extend_from_slice(&item);
        Ok(())
    }
}
// --- Implementation of SyncSequenceCodec ---

/// This is a wrapper around sending and receiving data from any async read/write source.
///
/// This allows us to use either a TCP, UDP, and Serial port in a generic way and change out the interface
/// to talk to the robot without changing the rest of the code.
pub struct DataInterface {
    /// The `Framed` data object
    framed: Framed<Box<dyn AsyncReadWrite>, SyncSequenceCodec>,
}

impl DataInterface {
    /// Creates a new `DataInterface` object from the given `AsyncReadWrite` object.
    /// This will create a `Framed` object with the `AnyDelimiterCodec` with the default delimiter of `DEFAULT_SYNC_BYTES` for both reading and writing.
    pub fn new(stream: Box<dyn AsyncReadWrite>) -> Self {
        let framed = Framed::new(stream, SyncSequenceCodec::new(DEFAULT_SYNC_BYTES.to_vec(), DEFAULT_SYNC_BYTES.to_vec()));
        DataInterface { framed }
    }

    /// Reads a response from the data interface.
    ///
    /// This will read a response from the data interface and return the bytes read, filtering out empty responses.
    pub async fn _read_frame(&mut self) -> Result<Vec<u8>, DataError> {
        let bytes = self
            .framed
            .borrow_mut()
            .try_filter(|v| future::ready(!v.is_empty()))
            .next()
            .await
            .ok_or(DataError::NoResponse)??
            .to_vec();
        Ok(bytes)
    }

    /// Writes the given bytes to the data interface.
    pub async fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), DataError> {
        self.framed.send(bytes.to_vec().into()).await?;
        Ok(())
    }

    /// Writes a VisionData packet to the data interface.
    pub async fn write_vision_data(&mut self, data: VisionData) -> Result<(), DataError> {
        let bytes = data.into_bytes();
        self.write_bytes(&bytes).await
    }
}
// --- Implementation of DataInterface ---
