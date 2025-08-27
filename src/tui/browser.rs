//! File browsing and pattern discovery for the TUI

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::LobachevskyError;

/// File browser for TOML pattern files
pub struct FileBrowser {
    /// Directory being browsed
    pub directory: PathBuf,
    /// Display name for this browser
    pub name: String,
    /// Available files
    pub files: Vec<FileEntry>,
    /// Currently selected index
    pub selected_index: usize,
}

/// Information about a discovered pattern file
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// File name without extension
    pub name: String,
    /// Full file path
    pub path: PathBuf,
    /// Pattern metadata loaded from file
    pub metadata: Option<PatternMetadata>,
}

/// Common metadata found in pattern files
#[derive(Debug, Clone, Deserialize)]
pub struct PatternMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tempo_hint: Option<u16>,
}

impl FileBrowser {
    pub fn new(directory: &str, name: &str) -> Result<Self, LobachevskyError> {
        let dir_path = PathBuf::from(directory);
        let mut browser = FileBrowser {
            directory: dir_path,
            name: name.to_string(),
            files: Vec::new(),
            selected_index: 0,
        };

        browser.scan_files()?;
        Ok(browser)
    }

    /// Scan the directory for TOML files and load their metadata
    fn scan_files(&mut self) -> Result<(), LobachevskyError> {
        self.files.clear();

        if !self.directory.exists() {
            return Ok(()); // Directory doesn't exist, but that's not fatal
        }

        let entries = fs::read_dir(&self.directory).map_err(|e| LobachevskyError::FileIo {
            path: self.directory.to_string_lossy().to_string(),
            source: e,
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| LobachevskyError::FileIo {
                path: self.directory.to_string_lossy().to_string(),
                source: e,
            })?;

            let path = entry.path();

            // Only process TOML files
            if path.extension().and_then(|ext| ext.to_str()) == Some("toml")
                && let Some(name) = path.file_stem().and_then(|stem| stem.to_str())
            {
                // Skip README files
                if name.to_lowercase() == "readme" {
                    continue;
                }

                let metadata = self.load_metadata(&path);

                self.files.push(FileEntry {
                    name: name.to_string(),
                    path: path.clone(),
                    metadata,
                });
            }
        }

        // Sort files alphabetically
        self.files.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(())
    }

    /// Load pattern metadata from a TOML file
    fn load_metadata(&self, path: &Path) -> Option<PatternMetadata> {
        match fs::read_to_string(path) {
            Ok(content) => {
                // Try to parse just the top-level metadata we care about
                toml::from_str::<PatternMetadata>(&content).ok()
            }
            Err(_) => None,
        }
    }

    /// Get the currently selected file
    pub fn selected_file(&self) -> Option<&str> {
        self.files.get(self.selected_index).map(|entry| entry.name.as_str())
    }

    /// Get the currently selected file entry
    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.files.get(self.selected_index)
    }

    /// Move selection down
    pub fn next(&mut self) {
        if self.selected_index < self.files.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Move selection up
    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Get display name for a file entry
    pub fn display_name(&self, entry: &FileEntry) -> String {
        if let Some(ref metadata) = entry.metadata
            && let Some(ref display_name) = metadata.name
        {
            return display_name.clone();
        }
        entry.name.clone()
    }

    /// Get description for a file entry
    pub fn description(&self, entry: &FileEntry) -> String {
        if let Some(ref metadata) = entry.metadata
            && let Some(ref desc) = metadata.description
        {
            return desc.clone();
        }
        "No description available".to_string()
    }

    /// Get tempo hint for a file entry
    pub fn tempo_hint(&self, entry: &FileEntry) -> Option<u16> {
        entry.metadata.as_ref().and_then(|m| m.tempo_hint)
    }

    /// Refresh the file list
    pub fn refresh(&mut self) -> Result<(), LobachevskyError> {
        let current_selection = self.selected_file().map(|s| s.to_string());
        self.scan_files()?;

        // Try to restore selection
        if let Some(name) = current_selection
            && let Some(index) = self.files.iter().position(|entry| entry.name == name)
        {
            self.selected_index = index;
        }

        // Ensure selection is valid
        if self.selected_index >= self.files.len() {
            self.selected_index = self.files.len().saturating_sub(1);
        }

        Ok(())
    }

    /// Get all file names for quick access
    pub fn file_names(&self) -> Vec<&str> {
        self.files.iter().map(|entry| entry.name.as_str()).collect()
    }

    /// Check if browser is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get count of available files
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}
