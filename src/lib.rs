use thiserror::Error;

pub mod channel_control;
pub mod commands;
pub mod fixed_channel_control;
pub mod spd3303x;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    ScpiClient(#[from] scpi_client::Error),
    #[error("Underlying I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to connect: {0}")]
    ConnectFailed(String),
    #[error("Serial mismatch: {0}")]
    SerialMismatch(String),
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Other: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
