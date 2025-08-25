//! Command input types for parsing CLI arguments into typed structures

use crate::generation::HexatonicCycle;
use crate::harmony::TypedHarmonicPattern;
use crate::melody::MelodyStrategy;
use crate::{Chord, LobachevskyError, Mode, PitchClass, Transform};

/// Input for the Progression command
pub struct ProgressionInput {
    pub start_chord: Chord,
    pub transforms: Vec<Transform>,
    pub length: usize,
    pub return_to_start: bool,
    pub mode: Option<Mode>,
    pub tonic: Option<PitchClass>,
}

impl ProgressionInput {
    pub fn from_cli(
        start: &str,
        pattern: &str,
        length: usize,
        return_to_start: bool,
        mode: Option<&str>,
        tonic: Option<&str>,
    ) -> Result<Self, LobachevskyError> {
        let start_chord = Chord::try_from(start)?;
        let transforms = parse_transforms(pattern)?;

        if transforms.is_empty() {
            return Err(LobachevskyError::ParseError {
                message: "Invalid transformation pattern: no transforms parsed".to_string(),
            });
        }

        let (parsed_mode, parsed_tonic) = parse_mode_and_tonic(mode, tonic)?;

        Ok(ProgressionInput {
            start_chord,
            transforms,
            length,
            return_to_start,
            mode: parsed_mode,
            tonic: parsed_tonic,
        })
    }
}

/// Input for the Hexatonic command
pub struct HexatonicInput {
    pub cycle: HexatonicCycle,
    pub start_chord: Chord,
}

impl HexatonicInput {
    pub fn from_cli(cycle: &str, start: &str) -> Result<Self, LobachevskyError> {
        let start_chord = Chord::try_from(start)?;

        let cycle = match cycle.to_lowercase().as_str() {
            "northern" => HexatonicCycle::Northern,
            "western" => HexatonicCycle::Western,
            "eastern" => HexatonicCycle::Eastern,
            _ => {
                return Err(LobachevskyError::ParseError {
                    message: format!("Invalid cycle type '{}'. Use: northern, western, or eastern", cycle),
                });
            }
        };

        Ok(HexatonicInput { cycle, start_chord })
    }
}

/// Input for the Modal command
pub struct ModalInput {
    pub mode: Mode,
    pub tonic: PitchClass,
    pub start_chord: Option<Chord>,
    pub transforms: Vec<Transform>,
    pub length: usize,
    pub analyze: bool,
}

impl ModalInput {
    pub fn from_cli(
        mode: &str,
        tonic: &str,
        start: Option<&str>,
        pattern: &str,
        length: usize,
        analyze: bool,
    ) -> Result<Self, LobachevskyError> {
        let mode = Mode::try_from(mode)?;
        let tonic = PitchClass::try_from(tonic)?;
        let transforms = parse_transforms(pattern)?;

        if transforms.is_empty() {
            return Err(LobachevskyError::ParseError {
                message: "Invalid transformation pattern: no transforms parsed".to_string(),
            });
        }

        let start_chord = if let Some(start_str) = start {
            Some(Chord::try_from(start_str)?)
        } else {
            None
        };

        Ok(ModalInput {
            mode,
            tonic,
            start_chord,
            transforms,
            length,
            analyze,
        })
    }
}

/// Input for the Extended command
pub struct ExtendedInput {
    pub start_chord: Chord,
    pub transforms: Vec<Transform>,
    pub length: usize,
    pub analyze: bool,
}

impl ExtendedInput {
    pub fn from_cli(start: &str, pattern: &str, length: usize, analyze: bool) -> Result<Self, LobachevskyError> {
        let start_chord = Chord::try_from(start)?;
        let transforms = parse_transforms(pattern)?;

        if transforms.is_empty() {
            return Err(LobachevskyError::ParseError {
                message: "Invalid transformation pattern: no transforms parsed".to_string(),
            });
        }

        Ok(ExtendedInput {
            start_chord,
            transforms,
            length,
            analyze,
        })
    }
}

/// Input for the Generate command
pub struct GenerateInput {
    pub rhythm_name: Option<String>,
    pub harmony: TypedHarmonicPattern,
    pub melody_strategy: MelodyStrategy,
    pub notes_per_chord: usize,
}

impl GenerateInput {
    #[allow(clippy::too_many_arguments)]
    pub fn from_cli(
        rhythm: Option<&str>,
        harmony_file: Option<&str>,
        start: &str,
        pattern: &str,
        mode: Option<&str>,
        tonic: Option<&str>,
        melody: &str,
        bars: usize,
        tempo: u16,
        notes_per_chord: usize,
        return_to_start: bool,
    ) -> Result<Self, LobachevskyError> {
        // Load or create harmonic pattern
        let harmony = if let Some(harmony_path) = harmony_file {
            // Load from file
            if harmony_path.ends_with(".toml") {
                let mut lib = crate::harmony::HarmonicLibrary::new();
                let pattern = lib.load_from_file(std::path::Path::new(harmony_path))?;
                pattern.to_typed()?
            } else {
                // Assume it's a pattern name from the harmonics directory
                let mut lib = crate::harmony::HarmonicLibrary::new();
                lib.load_from_directory(std::path::Path::new("harmonics"))?;
                let pattern = lib.get(harmony_path).ok_or_else(|| LobachevskyError::ParseError {
                    message: format!("Harmonic pattern '{}' not found", harmony_path),
                })?;
                pattern.to_typed()?
            }
        } else {
            // Create from command line arguments
            let start_chord = Chord::try_from(start)?;
            let transforms = parse_transforms(pattern)?;
            let (parsed_mode, parsed_tonic) = parse_mode_and_tonic(mode, tonic)?;

            TypedHarmonicPattern {
                name: "cli_generated".to_string(),
                description: Some("Generated from command line arguments".to_string()),
                start_chord,
                transformations: transforms,
                mode: parsed_mode,
                tonic: parsed_tonic,
                return_to_start,
                tempo_hint: Some(tempo),
                bars_hint: Some(bars),
            }
        };

        let melody_strategy = MelodyStrategy::from(melody);

        Ok(GenerateInput {
            rhythm_name: rhythm.map(|s| s.to_string()),
            harmony,
            melody_strategy,
            notes_per_chord,
        })
    }
}

// Helper functions

fn parse_transforms(pattern_str: &str) -> Result<Vec<Transform>, LobachevskyError> {
    pattern_str
        .split(',')
        .map(|s| Transform::try_from(s.trim()))
        .collect::<Result<Vec<_>, _>>()
}

fn parse_mode_and_tonic(
    mode: Option<&str>,
    tonic: Option<&str>,
) -> Result<(Option<Mode>, Option<PitchClass>), LobachevskyError> {
    match (mode, tonic) {
        (Some(mode_str), Some(tonic_str)) => {
            let parsed_mode = Mode::try_from(mode_str)?;
            let parsed_tonic = PitchClass::try_from(tonic_str)?;
            Ok((Some(parsed_mode), Some(parsed_tonic)))
        }
        (None, None) => Ok((None, None)),
        _ => Err(LobachevskyError::ParseError {
            message: "Both mode and tonic must be specified for modal constraints".to_string(),
        }),
    }
}
