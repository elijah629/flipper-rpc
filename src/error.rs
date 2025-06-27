//! Error type for all functions

use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive] // This may and will change as the crate approaches stable 1.0.0
/// Global error type for all rpc and io errors
pub enum Error {
    #[error("io: {0}")]
    /// An IO error, based on std::io::Error
    Io(#[from] std::io::Error),

    #[error("rpc: {0}")]
    #[cfg(feature = "easy-rpc")]
    /// An RPC error, based on rpc::error::Error
    Rpc(#[from] crate::rpc::error::Error),

    #[error("serialport: {0}")]
    #[cfg(feature = "transport-serial")]
    /// A Serialport error, based on serialport::Error
    Serialport(#[from] serialport::Error),

    #[error("prost: decode: {0}")]
    #[cfg(feature = "proto")]
    /// A protobuf decode error, based on prost::DecodeError
    ProtoDecode(#[from] prost::DecodeError),

    #[error("prost: encode: {0}")]
    #[cfg(feature = "proto")]
    /// A protobuf encode error, based on prost::EncodeError
    ProtoEncode(#[from] prost::EncodeError),

    #[error("mpsc: {0}")]
    #[cfg(feature = "fs-progress-mpsc")]
    /// MPSC Error in the storage module when using progress-mpsc
    MpscSend(#[from] std::sync::mpsc::SendError<usize>),
}

/// Result type based on error::Error
pub type Result<T> = std::result::Result<T, Error>;
