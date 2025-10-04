//! StoryStream Media Engine
//!
//! Audio playback engine for audiobooks

pub mod engine;
pub mod error;
pub mod types;

pub use engine::MediaEngine;
pub use error::{MediaError, Result};
pub use types::{Chapter, MediaEvent, PlaybackState, PlaybackStatus};

pub mod prelude {
    pub use crate::engine::MediaEngine;
    pub use crate::error::{MediaError, Result};
    pub use crate::types::{Chapter, MediaEvent, PlaybackState, PlaybackStatus};
}