//! Library and import configuration section

use crate::validation::{ConfigSection, ValidationError, Validator};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Library management and import settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LibraryConfig {
    /// Default library paths to scan for audiobooks
    pub library_paths: Vec<PathBuf>,

    /// Supported audio file extensions
    pub supported_extensions: Vec<String>,

    /// Auto-import new files when scanning
    pub auto_import: bool,

    /// Extract metadata from audio files
    pub extract_metadata: bool,

    /// Recurse into subdirectories when scanning
    pub recursive_scan: bool,

    /// Maximum recursion depth (0 = unlimited)
    pub max_scan_depth: u32,

    /// Skip files smaller than this size in bytes
    pub min_file_size_bytes: u64,

    /// Follow symbolic links when scanning
    pub follow_symlinks: bool,

    /// Organize imported files by author/title
    pub organize_files: bool,

    /// Target directory for organized files (if organize_files is true)
    pub organization_target: Option<PathBuf>,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            library_paths: Vec::new(),
            supported_extensions: vec![
                "mp3".to_string(),
                "m4a".to_string(),
                "m4b".to_string(),
                "ogg".to_string(),
                "opus".to_string(),
                "flac".to_string(),
                "wav".to_string(),
            ],
            auto_import: false,
            extract_metadata: true,
            recursive_scan: true,
            max_scan_depth: 0, // unlimited
            min_file_size_bytes: 1024, // 1 KB
            follow_symlinks: false,
            organize_files: false,
            organization_target: None,
        }
    }
}

impl ConfigSection for LibraryConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut results = vec![Validator::in_range(
            self.min_file_size_bytes,
            0,
            100 * 1024 * 1024, // 100 MB max
            "library.min_file_size_bytes",
        )];

        // Validate that extensions are not empty
        for (i, ext) in self.supported_extensions.iter().enumerate() {
            results.push(Validator::not_empty(
                ext,
                &format!("library.supported_extensions[{}]", i),
            ));
        }

        // Validate organization target if organize_files is enabled
        if self.organize_files {
            if let Some(ref target) = self.organization_target {
                // We don't validate path existence here because the directory
                // might not exist yet and will be created
                if target.as_os_str().is_empty() {
                    results.push(Err(ValidationError::new(
                        "library.organization_target",
                        "must not be empty when organize_files is true",
                    )));
                }
            } else {
                results.push(Err(ValidationError::new(
                    "library.organization_target",
                    "must be set when organize_files is true",
                )));
            }
        }

        Validator::collect_errors(results)
    }

    fn merge(&mut self, other: Self) {
        self.library_paths = other.library_paths;
        self.supported_extensions = other.supported_extensions;
        self.auto_import = other.auto_import;
        self.extract_metadata = other.extract_metadata;
        self.recursive_scan = other.recursive_scan;
        self.max_scan_depth = other.max_scan_depth;
        self.min_file_size_bytes = other.min_file_size_bytes;
        self.follow_symlinks = other.follow_symlinks;
        self.organize_files = other.organize_files;
        self.organization_target = other.organization_target;
    }

    fn section_name(&self) -> &'static str {
        "library"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_valid() {
        let config = LibraryConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_min_file_size() {
        let mut config = LibraryConfig::default();
        config.min_file_size_bytes = 200 * 1024 * 1024; // 200 MB - too large
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_extension() {
        let mut config = LibraryConfig::default();
        config.supported_extensions.push("".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_organize_files_without_target() {
        let mut config = LibraryConfig::default();
        config.organize_files = true;
        config.organization_target = None;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_organize_files_with_empty_target() {
        let mut config = LibraryConfig::default();
        config.organize_files = true;
        config.organization_target = Some(PathBuf::new());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_organize_files_with_valid_target() {
        let mut config = LibraryConfig::default();
        config.organize_files = true;
        config.organization_target = Some(PathBuf::from("/valid/path"));
        // Validation should pass even if path doesn't exist
        // (it will be created when needed)
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_merge() {
        let mut base = LibraryConfig::default();
        let mut other = LibraryConfig::default();
        other.auto_import = true;
        other.extract_metadata = false;
        other.library_paths = vec![PathBuf::from("/books")];

        base.merge(other);
        assert!(base.auto_import);
        assert!(!base.extract_metadata);
        assert_eq!(base.library_paths, vec![PathBuf::from("/books")]);
    }
}