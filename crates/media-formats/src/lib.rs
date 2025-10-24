extern crate core;

mod capabilities;
mod detection;
mod error;
mod format;
mod mime;
mod properties;
mod quality;

// Re-export all types
pub use capabilities::{FormatCapabilities, MetadataSupport, QualityLevel};
pub use detection::FormatDetector;
pub use error::{FormatError, FormatResult};
pub use format::AudioFormat;
pub use mime::MimeType;
pub use properties::{AudioAnalyzer, AudioProperties, CodecInfo};
pub use quality::{AudioQuality, QualityTier};

pub mod prelude {
    pub use crate::{
        AudioAnalyzer, AudioFormat, AudioProperties, AudioQuality, FormatCapabilities,
        FormatDetector, FormatError, FormatResult, QualityTier,
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
