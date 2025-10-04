use thiserror::Error;

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("Failed to load audio file: {0}")]
    LoadError(String),

    #[error("Failed to decode audio: {0}")]
    DecodeError(String),

    #[error("Playback error: {0}")]
    PlaybackError(String),

    #[error("Invalid seek position: {0}")]
    InvalidSeek(String),

    #[error("Invalid playback speed: {0}. Must be between 0.25 and 3.0")]
    InvalidSpeed(f32),

    #[error("Chapter not found: {0}")]
    ChapterNotFound(usize),

    #[error("No media loaded")]
    NoMediaLoaded,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Audio stream error: {0}")]
    StreamError(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Engine not initialized")]
    NotInitialized,
}

pub type Result<T> = std::result::Result<T, MediaError>;