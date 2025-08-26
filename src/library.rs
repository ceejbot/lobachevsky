//! A pattern library. The timeless way of building.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::LobachevskyError;
use crate::harmony::HarmonicPattern;
use crate::rhythm::PatternData;

/// Trait for types that can be used in a library (have a name and can be
/// deserialized from TOML)
pub trait LibraryItem {
    fn name(&self) -> &str;
}

impl LibraryItem for HarmonicPattern {
    fn name(&self) -> &str {
        &self.name
    }
}

impl LibraryItem for PatternData {
    fn name(&self) -> &str {
        &self.name
    }
}

pub trait Library<'de> {
    type T: Deserialize<'de> + Serialize + std::fmt::Debug;

    fn load_all_from(&mut self, path: &Path) -> Result<usize, LobachevskyError>;
    fn load_one(&mut self, fpath: &Path) -> Result<Self::T, LobachevskyError>;
    fn get(&self, name: &str) -> Option<&Self::T>;
    fn add(&mut self, item: Self::T);
    fn list(&self) -> Vec<&str>;
}

pub const PATTERN_LIB: &str = "library/patterns";
pub const HARMONICS_LIB: &str = "library/harmonics";

// Type aliases for specific library types
pub type HarmonicLibrary = Librarian<HarmonicPattern>;
pub type PatternLibrary = Librarian<PatternData>;

// Convenience implementations for the type aliases
impl HarmonicLibrary {
    /// Load patterns from a directory (convenience method)
    pub fn load_from_directory(&mut self, path: &Path) -> Result<usize, LobachevskyError> {
        self.load_all_from(path)
    }

    /// Load a single pattern from a file (convenience method)
    pub fn load_from_file(&mut self, path: &Path) -> Result<HarmonicPattern, LobachevskyError> {
        self.load_one(path)
    }
}

impl PatternLibrary {
    /// Load patterns from a directory (convenience method)
    pub fn load_from_directory(&mut self, path: &Path) -> Result<(), LobachevskyError> {
        self.load_all_from(path).map(|_| ())
    }
}

#[derive(Debug, Clone)]
pub struct Librarian<P>
where
    P: for<'de> Deserialize<'de> + Serialize + std::fmt::Debug + LibraryItem + Clone,
{
    patterns: HashMap<String, P>,
}

impl<P> Librarian<P>
where
    P: for<'de> Deserialize<'de> + Serialize + std::fmt::Debug + LibraryItem + Clone,
{
    pub fn new() -> Self {
        Librarian {
            patterns: HashMap::new(),
        }
    }
}

impl<P> Default for Librarian<P>
where
    P: for<'de> Deserialize<'de> + Serialize + std::fmt::Debug + LibraryItem + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, P> Library<'de> for Librarian<P>
where
    P: for<'a> Deserialize<'a> + Serialize + std::fmt::Debug + LibraryItem + Clone,
{
    type T = P;

    fn load_all_from(&mut self, fpath: &Path) -> Result<usize, LobachevskyError> {
        let entries = std::fs::read_dir(fpath).map_err(|source| LobachevskyError::DirectoryReadError {
            path: fpath.to_path_buf(),
            source,
        })?;

        for entry in entries {
            let entry = entry.map_err(|source| LobachevskyError::DirectoryReadError {
                path: fpath.to_path_buf(),
                source,
            })?;
            let file_path = entry.path();

            if file_path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let contents =
                    std::fs::read_to_string(&file_path).map_err(|source| LobachevskyError::PatternFileError {
                        path: file_path.clone(),
                        source,
                    })?;

                let pattern: P =
                    toml::from_str(&contents).map_err(|source| LobachevskyError::PatternParseError { source })?;
                self.patterns.insert(pattern.name().to_string(), pattern);
            }
        }

        Ok(self.patterns.len())
    }

    fn load_one(&mut self, fpath: &Path) -> Result<Self::T, LobachevskyError> {
        let contents = std::fs::read_to_string(fpath).map_err(|source| LobachevskyError::PatternFileError {
            path: fpath.to_path_buf(),
            source,
        })?;

        let pattern: P = toml::from_str(&contents).map_err(|source| LobachevskyError::PatternParseError { source })?;

        self.patterns.insert(pattern.name().to_string(), pattern.clone());
        Ok(pattern)
    }

    fn get(&self, name: &str) -> Option<&Self::T> {
        self.patterns.get(name)
    }

    fn add(&mut self, item: Self::T) {
        self.patterns.insert(item.name().to_string(), item);
    }

    fn list(&self) -> Vec<&str> {
        self.patterns.keys().map(|s| s.as_str()).collect()
    }
}
