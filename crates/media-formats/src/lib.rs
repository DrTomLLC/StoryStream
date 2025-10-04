// crates/media-formats/src/lib.rs
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    // Lossless
    FLAC,
    ALAC,
    WAV,
    AIFF,
    WavPack,
    APE,

    // High-Quality Lossy
    Opus,
    Vorbis,
    AAC,      // M4A
    M4B,      // Audiobook with chapters
    MP3,

    // Unknown
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioQuality {
    /// 16-bit, 44.1kHz (CD quality)
    CD,
    /// 16-bit, 48kHz (DVD quality)
    DVD,
    /// 24-bit, 96kHz (Hi-Res)
    HiRes96,
    /// 24-bit, 192kHz (Ultra Hi-Res)
    HiRes192,
    /// 32-bit float (Studio)
    Studio,
}

#[derive(Debug, Clone)]
pub struct AudioProperties {
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub bits_per_sample: u8,
    pub channels: u8,
    pub bitrate: Option<u32>, // For lossy formats
    pub is_lossless: bool,
    pub is_variable_bitrate: bool,
}

impl AudioFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| Self::from_extension(ext))
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "flac" => Some(Self::FLAC),
            "m4a" => Some(Self::AAC),
            "m4b" => Some(Self::M4B),
            "mp3" => Some(Self::MP3),
            "ogg" | "oga" => Some(Self::Vorbis),
            "opus" => Some(Self::Opus),
            "wav" => Some(Self::WAV),
            "aiff" | "aif" => Some(Self::AIFF),
            "wv" => Some(Self::WavPack),
            "ape" => Some(Self::APE),
            "alac" => Some(Self::ALAC),
            _ => None,
        }
    }

    pub fn is_lossless(&self) -> bool {
        matches!(
            self,
            Self::FLAC | Self::ALAC | Self::WAV | Self::AIFF | Self::WavPack | Self::APE
        )
    }

    pub fn supports_chapters(&self) -> bool {
        matches!(self, Self::M4B | Self::FLAC | Self::ALAC)
    }
}