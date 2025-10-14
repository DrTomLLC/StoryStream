use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    Io(std::io::Error),
    Decode(String),
    DecodeError(String),
    InvalidSpeed(f32),
    SeekError(String),
    OutputError(String),
    InvalidState(String),
    Other(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Decode(e) => write!(f, "Decode error: {}", e),
            Self::DecodeError(e) => write!(f, "Decode error: {}", e),
            Self::InvalidSpeed(s) => write!(f, "Invalid speed: {}", s),
            Self::SeekError(e) => write!(f, "Seek error: {}", e),
            Self::OutputError(e) => write!(f, "Output error: {}", e),
            Self::InvalidState(e) => write!(f, "Invalid state: {}", e),
            Self::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<std::io::Error> for EngineError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub type EngineResult<T> = std::result::Result<T, EngineError>;