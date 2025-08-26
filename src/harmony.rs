//! Harmonic pattern definitions and library for TOML serialization

use serde::{Deserialize, Serialize};

use crate::{Chord, LobachevskyError, Mode, PitchClass, Transform};

/// Typed harmonic pattern with parsed types for internal use
#[derive(Debug, Clone)]
pub struct TypedHarmonicPattern {
    /// Name of the harmonic pattern
    pub name: String,

    /// Description of the pattern
    pub description: Option<String>,

    /// Starting chord (parsed)
    pub start_chord: Chord,

    /// List of transformations to apply (parsed)
    pub transformations: Vec<Transform>,

    /// Optional mode constraint (parsed)
    pub mode: Option<Mode>,

    /// Modal tonic if mode is specified (parsed)
    pub tonic: Option<PitchClass>,

    /// Whether to return to the starting chord
    pub return_to_start: bool,

    /// Suggested tempo for this progression
    pub tempo_hint: Option<u16>,

    /// Suggested number of bars
    pub bars_hint: Option<usize>,
}

/// Data structure for harmonic progressions that can be serialized to TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicPattern {
    /// Name of the harmonic pattern
    pub name: String,

    /// Description of the pattern
    pub description: Option<String>,

    /// Starting chord (e.g., "Am", "C", "F#m")
    pub start_chord: String,

    /// List of transformations to apply
    pub transformations: Vec<String>,

    /// Optional mode constraint (e.g., "dorian", "lydian")
    pub mode: Option<String>,

    /// Modal tonic if mode is specified (e.g., "C", "D")
    pub tonic: Option<String>,

    /// Whether to return to the starting chord
    pub return_to_start: Option<bool>,

    /// Suggested tempo for this progression
    pub tempo_hint: Option<u16>,

    /// Suggested number of bars
    pub bars_hint: Option<usize>,
}

impl Default for HarmonicPattern {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: Some("the default harmonic pattern".to_string()),
            start_chord: "Cmaj".to_string(),
            transformations: Default::default(),
            mode: None,
            tonic: None,
            return_to_start: None,
            tempo_hint: None,
            bars_hint: None,
        }
    }
}

impl HarmonicPattern {
    /// Load pattern from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, LobachevskyError> {
        toml::from_str(toml_str).map_err(|source| LobachevskyError::PatternParseError { source })
    }

    /// Save pattern to TOML string
    pub fn to_toml(&self) -> Result<String, LobachevskyError> {
        toml::to_string_pretty(self).map_err(|_e| LobachevskyError::ParseError {
            message: "Failed to serialize pattern to TOML".to_string(),
        })
    }

    /// Parse transformations into Transform enum
    pub fn parse_transformations(&self) -> Result<Vec<Transform>, LobachevskyError> {
        self.transformations
            .iter()
            .map(|t| Transform::try_from(t.as_str()))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Convert to a typed harmonic pattern with all strings parsed to types
    pub fn to_typed(&self) -> Result<TypedHarmonicPattern, LobachevskyError> {
        let start_chord = Chord::try_from(self.start_chord.as_str())?;
        let transformations = self.parse_transformations()?;

        let mode = if let Some(mode_str) = &self.mode {
            Some(Mode::try_from(mode_str.as_str())?)
        } else {
            None
        };

        let tonic = if let Some(tonic_str) = &self.tonic {
            Some(PitchClass::try_from(tonic_str.as_str())?)
        } else {
            None
        };

        // Validate that mode and tonic are both present or both absent
        if mode.is_some() != tonic.is_some() {
            return Err(LobachevskyError::ParseError {
                message: "Mode and tonic must both be specified or both be absent".to_string(),
            });
        }

        Ok(TypedHarmonicPattern {
            name: self.name.clone(),
            description: self.description.clone(),
            start_chord,
            transformations,
            mode,
            tonic,
            return_to_start: self.return_to_start.unwrap_or(false),
            tempo_hint: self.tempo_hint,
            bars_hint: self.bars_hint,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::library::*;

    #[test]
    fn can_parse_harmonic_pattern() {
        let toml_str = r#"
            name = "test_pattern"
            description = "A test pattern"
            start_chord = "Am"
            transformations = ["P", "L", "R"]
            return_to_start = true
            tempo_hint = 120
            bars_hint = 16
        "#;

        let pattern = HarmonicPattern::from_toml(toml_str).expect("Should parse TOML");
        assert_eq!(pattern.name, "test_pattern");
        assert_eq!(pattern.start_chord, "Am");
        assert_eq!(pattern.transformations, vec!["P", "L", "R"]);
        assert_eq!(pattern.return_to_start, Some(true));
        assert_eq!(pattern.tempo_hint, Some(120));
        assert_eq!(pattern.bars_hint, Some(16));
    }

    #[test]
    fn parsing_harmonic_pattern_transform() {
        let pattern = HarmonicPattern {
            name: "test".to_string(),
            description: None,
            start_chord: "C".to_string(),
            transformations: vec!["P".to_string(), "L".to_string(), "R".to_string(), "PL".to_string()],
            mode: None,
            tonic: None,
            return_to_start: None,
            tempo_hint: None,
            bars_hint: None,
        };

        let transforms = pattern.parse_transformations().expect("Should parse transforms");
        assert_eq!(transforms.len(), 4);
    }

    #[test]
    fn harmonic_pattern_toml_roundtrip() {
        let pattern = HarmonicPattern {
            name: "test_pattern".to_string(),
            description: Some("A test harmonic pattern".to_string()),
            start_chord: "Am".to_string(),
            transformations: vec!["P".to_string(), "L".to_string(), "R".to_string()],
            mode: Some("dorian".to_string()),
            tonic: Some("D".to_string()),
            return_to_start: Some(true),
            tempo_hint: Some(120),
            bars_hint: Some(32),
        };

        // Convert to TOML and back
        let toml_str = pattern.to_toml().expect("Should serialize to TOML");
        let loaded = HarmonicPattern::from_toml(&toml_str).expect("Should parse from TOML");

        assert_eq!(loaded.name, pattern.name);
        assert_eq!(loaded.start_chord, pattern.start_chord);
        assert_eq!(loaded.transformations, pattern.transformations);
        assert_eq!(loaded.mode, pattern.mode);
        assert_eq!(loaded.tonic, pattern.tonic);
    }

    #[test]
    fn parse_transformations() {
        let pattern = HarmonicPattern {
            name: "test".to_string(),
            description: None,
            start_chord: "C".to_string(),
            transformations: vec!["P".to_string(), "R".to_string(), "L".to_string()],
            mode: None,
            tonic: None,
            return_to_start: None,
            tempo_hint: None,
            bars_hint: None,
        };

        let transforms = pattern.parse_transformations().expect("Should parse transforms");
        assert_eq!(transforms.len(), 3);
        assert_eq!(transforms[0], Transform::P);
        assert_eq!(transforms[1], Transform::R);
        assert_eq!(transforms[2], Transform::L);
    }

    #[test]
    fn harmonic_library_works() {
        let mut library = crate::library::HarmonicLibrary::new();

        let pattern = HarmonicPattern {
            name: "test_pattern".to_string(),
            description: Some("Test".to_string()),
            start_chord: "F".to_string(),
            transformations: vec!["P".to_string(), "L".to_string()],
            mode: None,
            tonic: None,
            return_to_start: Some(true),
            tempo_hint: Some(110),
            bars_hint: Some(32),
        };

        library.add(pattern.clone());

        let retrieved = library.get("test_pattern").expect("test_pattern should exist");
        assert_eq!(retrieved.name, "test_pattern");
        assert_eq!(retrieved.start_chord, "F");
    }

    #[test]
    fn loading_harmonic_library() {
        // This test assumes the harmonics directory exists with our test patterns
        if Path::new(crate::library::HARMONICS_LIB).exists() {
            let mut library = crate::library::HarmonicLibrary::new();
            let result = library.load_from_directory(Path::new(crate::library::HARMONICS_LIB));

            match result {
                Ok(_) => {
                    let patterns = library.list();
                    assert!(!patterns.is_empty(), "Should load some patterns");

                    // Check for our known patterns
                    if library.get("classic_plr").is_some() {
                        let classic = library.get("classic_plr").expect("classic_plr should exist");
                        assert_eq!(classic.start_chord, "C");
                    }
                }
                Err(e) => {
                    log::info!("Warning: Could not load harmonics directory: {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_modal_harmonic_pattern() {
        let toml_str = r#"
            name = "modal_test"
            description = "Test modal constraints"
            start_chord = "Dm"
            transformations = ["P", "L", "R"]
            mode = "dorian"
            tonic = "D"
            return_to_start = true
        "#;

        let pattern = HarmonicPattern::from_toml(toml_str).expect("Should parse modal pattern");
        assert_eq!(pattern.mode, Some("dorian".to_string()));
        assert_eq!(pattern.tonic, Some("D".to_string()));
    }

    #[test]
    fn test_compound_transformations() {
        let pattern = HarmonicPattern {
            name: "compound_test".to_string(),
            description: None,
            start_chord: "C".to_string(),
            transformations: vec!["PL".to_string(), "RP".to_string(), "LPL".to_string()],
            mode: None,
            tonic: None,
            return_to_start: None,
            tempo_hint: None,
            bars_hint: None,
        };

        let transforms = pattern
            .parse_transformations()
            .expect("Should parse compound transforms");
        assert_eq!(transforms.len(), 3);

        // The compound transformations should be parsed correctly
        // This tests that our Transform::try_from handles compound transforms
    }
}
