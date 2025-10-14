extern crate core;

mod capabilities;
mod detection;
mod format;
mod mime;
mod error;
mod quality;
mod properties;

// Re-export all types
pub use capabilities::{FormatCapabilities, MetadataSupport, QualityLevel};
pub use detection::FormatDetector;
pub use format::AudioFormat;
pub use mime::MimeType;
pub use error::{FormatError, FormatResult};
pub use quality::{AudioQuality, QualityTier};
pub use properties::{AudioProperties, AudioAnalyzer, CodecInfo};

pub mod prelude {
    pub use crate::{
        AudioAnalyzer, AudioFormat, AudioProperties, AudioQuality,
        FormatCapabilities, FormatDetector, FormatError, FormatResult,
        QualityTier,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_modules_compile() {
        let _ = AudioFormat::Mp3;
        let _ = FormatCapabilities::for_format(AudioFormat::Mp3);
        let _ = FormatDetector::new();
        let _ = MimeType::from_format(AudioFormat::Mp3);
    }
}