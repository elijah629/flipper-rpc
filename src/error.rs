use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    SerialPort(#[from] serialport::Error),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    DecodeError(#[from] prost::DecodeError),
}

pub type Result<T> = std::result::Result<T, Error>;
