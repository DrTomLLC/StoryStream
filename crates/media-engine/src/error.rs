// FILE: crates/media-engine/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Decode error: {0}")]
    DecodeError(String),

    #[error("Output error: {0}")]
    OutputError(String),

    #[error("Seek error: {0}")]
    SeekError(String),

    #[error("Invalid speed: {0}")]
    InvalidSpeed(f32),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type EngineResult<T> = Result<T, EngineError>;