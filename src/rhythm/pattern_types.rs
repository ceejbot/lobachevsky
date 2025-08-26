//! The pattern data and library read from toml files.

use serde::{Deserialize, Serialize};

use super::*;
use crate::LobachevskyError;

/// Data structure for defining patterns in a declarative way
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternData {
    pub name: String,
    pub description: Option<String>,
    pub tempo_hint: Option<u16>,
    pub tracks: Vec<LayerData>,
    #[serde(default)]
    pub bass_tracks: Vec<BassLayerData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerData {
    pub voice: String,
    pub pattern_type: PatternType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PatternType {
    Euclidean {
        hits: usize,
        steps: usize,
        rotation: Option<usize>,
        velocity: Option<u8>,
    },
    Probability {
        points: Vec<ProbabilityPoint>,
        velocity_range: Option<(u8, u8)>,
    },
    Swing {
        /// Base pattern to apply swing to
        base_pattern: Box<PatternType>,
        /// Swing ratio: 0.5 = straight, 0.67 = heavy swing, 0.75 = very heavy
        swing_ratio: f64,
        /// Subdivision for swing: 0.25 = 16th notes, 0.5 = 8th notes
        subdivision: f64,
        /// Optional velocity variation for swung beats
        swing_accent: Option<u8>,
    },
    Polyrhythmic {
        /// Time signature (numerator, denominator)
        time_signature: (u16, u16),
        /// Number of bars before pattern repeats
        pattern_length: u16,
        /// Base pattern to apply polyrhythmic timing to
        base_pattern: Box<PatternType>,
    },
    Groove {
        /// Base pattern to apply groove to
        base_pattern: Box<PatternType>,
        /// Groove template name or custom timing adjustments
        groove_type: GrooveType,
        /// Intensity of groove effect (0.0 to 1.0)
        intensity: f64,
        /// Amount of random humanization
        humanization: Option<f64>,
    },
    /// Isochronic tones for brainwave entrainment
    Isochronic {
        /// Pulse frequency in Hz (e.g. 10.0 for 10 Hz alpha waves)
        frequency_hz: f64,
        /// Pulse velocity (0-127)
        velocity: Option<u8>,
        /// Base pattern to layer isochronic pulses over (optional)
        base_pattern: Option<Box<PatternType>>,
        /// Brainwave category for documentation
        brainwave_type: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbabilityPoint {
    pub beat: f64,
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BassLayerData {
    pub voice: String, // Maps to BassVoice enum
    pub pattern: BassPatternType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BassPatternType {
    Euclidean {
        hits: usize,
        steps: usize,
        rotation: Option<usize>,
        note_pattern: Vec<String>, // ["root", "fifth", "octave"]
        velocity: Option<u8>,
    },
    Probability {
        points: Vec<BassProbabilityPoint>,
        velocity_range: Option<(u8, u8)>,
    },
    Sequence {
        sequence: Vec<String>, // ["root", "root", "fifth", "octave"]
        note_duration: f64,
        velocity_pattern: Option<Vec<u8>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BassProbabilityPoint {
    pub beat: f64,
    pub probability: f64,
    pub note: String, // "root", "third", "fifth", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "groove_type")]
pub enum GrooveType {
    /// Named groove templates
    Template { name: String },
    /// Custom microtiming adjustments per beat
    Custom {
        /// Map of beat positions to timing adjustments (in beats)
        timing_map: std::collections::HashMap<String, f64>,
        /// Map of beat positions to velocity multipliers
        velocity_map: Option<std::collections::HashMap<String, f64>>,
    },
}

impl PatternData {
    /// Load pattern from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, LobachevskyError> {
        toml::from_str(toml_str).map_err(|source| LobachevskyError::PatternParseError { source })
    }

    /// Convert to RhythmPattern
    pub fn to_pattern(&self) -> Result<Box<dyn RhythmPattern>, LobachevskyError> {
        let mut layered = LayeredPattern::new();

        for layer in &self.tracks {
            let voice = parse_voice(&layer.voice)?;
            let pattern: Box<dyn RhythmPattern> = match &layer.pattern_type {
                PatternType::Euclidean {
                    hits,
                    steps,
                    rotation,
                    velocity,
                } => {
                    let mut euclidean = EuclideanPattern::new(voice, *hits, *steps);
                    if let Some(rot) = rotation {
                        euclidean = euclidean.with_rotation(*rot);
                    }
                    if let Some(vel) = velocity {
                        euclidean = euclidean.with_velocity(*vel);
                    }
                    Box::new(euclidean)
                }
                PatternType::Probability { points, velocity_range } => {
                    let mut prob = ProbabilityPattern::new(voice);
                    for point in points {
                        prob = prob.add_point(Beat(point.beat), point.probability);
                    }
                    if let Some((min, max)) = velocity_range {
                        prob.velocity_range = (*min, *max);
                    }
                    Box::new(prob)
                }
                PatternType::Swing {
                    base_pattern,
                    swing_ratio,
                    subdivision,
                    swing_accent,
                } => {
                    let base = pattern_type_to_rhythm(base_pattern, voice)?;
                    let mut swing = SwingPattern::new(base, *swing_ratio, *subdivision);
                    if let Some(accent) = swing_accent {
                        swing = swing.with_swing_accent(*accent);
                    }
                    Box::new(swing)
                }
                PatternType::Polyrhythmic {
                    time_signature,
                    pattern_length,
                    base_pattern,
                } => {
                    let base = pattern_type_to_rhythm(base_pattern, voice)?;
                    let polyrhythm = PolyrhythmicPattern::new(base, *time_signature, *pattern_length);
                    Box::new(polyrhythm)
                }
                PatternType::Groove {
                    base_pattern,
                    groove_type,
                    intensity,
                    humanization,
                } => {
                    let base = pattern_type_to_rhythm(base_pattern, voice)?;
                    let mut groove = GroovePattern::new(base, groove_type.clone(), *intensity);
                    if let Some(humanization_amount) = humanization {
                        groove = groove.with_humanization(*humanization_amount);
                    }
                    Box::new(groove)
                }
                PatternType::Isochronic {
                    frequency_hz,
                    velocity,
                    base_pattern,
                    brainwave_type: _,
                } => {
                    let mut isochronic = IsochronicPattern::new(voice, *frequency_hz);
                    if let Some(vel) = velocity {
                        isochronic = isochronic.with_velocity(*vel);
                    }
                    if let Some(base) = base_pattern {
                        let base_rhythm = pattern_type_to_rhythm(base, voice)?;
                        isochronic = isochronic.with_base_pattern(base_rhythm);
                    }
                    Box::new(isochronic)
                }
            };
            layered = layered.add_layer(pattern);
        }

        Ok(Box::new(layered))
    }

    /// Convert bass tracks to BassPattern implementations
    pub fn to_bass_patterns(&self) -> Result<Vec<Box<dyn BassPattern>>, LobachevskyError> {
        let mut patterns = Vec::new();

        for bass_track in &self.bass_tracks {
            let voice = parse_bass_voice(&bass_track.voice)?;
            let pattern: Box<dyn BassPattern> = match &bass_track.pattern {
                BassPatternType::Euclidean {
                    hits,
                    steps,
                    rotation,
                    note_pattern,
                    velocity,
                } => {
                    let mut euclidean = EuclideanBassPattern::new(voice, *hits, *steps);
                    if let Some(rot) = rotation {
                        euclidean = euclidean.with_rotation(*rot);
                    }
                    if let Some(vel) = velocity {
                        euclidean = euclidean.with_velocity(*vel);
                    }
                    euclidean = euclidean.with_note_pattern(note_pattern.clone());
                    Box::new(euclidean)
                }
                BassPatternType::Probability { points, velocity_range } => {
                    let mut prob = ProbabilityBassPattern::new(voice);
                    for point in points {
                        prob = prob.add_point(Beat(point.beat), point.probability, point.note.clone());
                    }
                    if let Some((min, max)) = velocity_range {
                        prob.velocity_range = (*min, *max);
                    }
                    Box::new(prob)
                }
                BassPatternType::Sequence {
                    sequence,
                    note_duration,
                    velocity_pattern,
                } => {
                    let mut seq = SequenceBassPattern::new(voice, sequence.clone());
                    seq = seq.with_note_duration(*note_duration);
                    if let Some(pattern) = velocity_pattern {
                        seq = seq.with_velocity_pattern(pattern.clone());
                    }
                    Box::new(seq)
                }
            };
            patterns.push(pattern);
        }

        Ok(patterns)
    }
}

fn parse_voice(voice_str: &str) -> Result<DrumVoice, LobachevskyError> {
    DrumVoice::try_from(voice_str)
}

fn parse_bass_voice(voice_str: &str) -> Result<BassVoice, LobachevskyError> {
    match voice_str.to_lowercase().as_str() {
        "bass" => Ok(BassVoice::Bass),
        "bass_staccato" | "bassstaccato" => Ok(BassVoice::BassStaccato),
        "bass_legato" | "basslegato" => Ok(BassVoice::BassLegato),
        "bass_sub" | "basssub" | "sub" => Ok(BassVoice::BassSub),
        "bass_slide" | "bassslide" | "slide" => Ok(BassVoice::BassSlide),
        _ => Err(LobachevskyError::ParseError {
            message: format!("Unknown bass voice: {}", voice_str),
        }),
    }
}

/// Helper method to convert a PatternType to a RhythmPattern (for nested
/// patterns)
fn pattern_type_to_rhythm(
    pattern_type: &PatternType,
    voice: DrumVoice,
) -> Result<Box<dyn RhythmPattern>, LobachevskyError> {
    match pattern_type {
        PatternType::Euclidean {
            hits,
            steps,
            rotation,
            velocity,
        } => {
            let mut euclidean = EuclideanPattern::new(voice, *hits, *steps);
            if let Some(rot) = rotation {
                euclidean = euclidean.with_rotation(*rot);
            }
            if let Some(vel) = velocity {
                euclidean = euclidean.with_velocity(*vel);
            }
            Ok(Box::new(euclidean))
        }
        PatternType::Probability { points, velocity_range } => {
            let mut prob = ProbabilityPattern::new(voice);
            for point in points {
                prob = prob.add_point(Beat(point.beat), point.probability);
            }
            if let Some((min, max)) = velocity_range {
                prob.velocity_range = (*min, *max);
            }
            Ok(Box::new(prob))
        }
        PatternType::Swing {
            base_pattern,
            swing_ratio,
            subdivision,
            swing_accent,
        } => {
            let base = pattern_type_to_rhythm(base_pattern, voice)?;
            let mut swing = SwingPattern::new(base, *swing_ratio, *subdivision);
            if let Some(accent) = swing_accent {
                swing = swing.with_swing_accent(*accent);
            }
            Ok(Box::new(swing))
        }
        PatternType::Polyrhythmic {
            time_signature,
            pattern_length,
            base_pattern,
        } => {
            let base = pattern_type_to_rhythm(base_pattern, voice)?;
            let polyrhythm = PolyrhythmicPattern::new(base, *time_signature, *pattern_length);
            Ok(Box::new(polyrhythm))
        }
        PatternType::Groove {
            base_pattern,
            groove_type,
            intensity,
            humanization,
        } => {
            let base = pattern_type_to_rhythm(base_pattern, voice)?;
            let mut groove = GroovePattern::new(base, groove_type.clone(), *intensity);
            if let Some(humanization_amount) = humanization {
                groove = groove.with_humanization(*humanization_amount);
            }
            Ok(Box::new(groove))
        }
        PatternType::Isochronic {
            frequency_hz,
            velocity,
            base_pattern,
            brainwave_type: _,
        } => {
            let mut isochronic = IsochronicPattern::new(voice, *frequency_hz);
            if let Some(vel) = velocity {
                isochronic = isochronic.with_velocity(*vel);
            }
            if let Some(base) = base_pattern {
                let base_rhythm = pattern_type_to_rhythm(base, voice)?;
                isochronic = isochronic.with_base_pattern(base_rhythm);
            }
            Ok(Box::new(isochronic))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bass_pattern_loading() {
        let toml_str = r#"
            name = "test_bass"
            description = "Test bass pattern"
            tempo_hint = 120

            [[tracks]]
            voice = "kick"
            [tracks.pattern_type]
            type = "Euclidean"
            hits = 4
            steps = 4
            velocity = 100

            [[bass_tracks]]
            voice = "bass"
            [bass_tracks.pattern]
            type = "Sequence"
            sequence = ["root", "fifth", "octave"]
            note_duration = 0.25
            velocity_pattern = [80, 75, 85]

            [[bass_tracks]]
            voice = "bass_staccato"
            [bass_tracks.pattern]
            type = "Probability"
            points = [
                { beat = 0.5, probability = 0.8, note = "fifth" },
                { beat = 2.5, probability = 0.7, note = "third" },
            ]
            velocity_range = [60, 80]
        "#;

        let pattern_data = PatternData::from_toml(toml_str).expect("Should parse TOML");
        assert_eq!(pattern_data.name, "test_bass");
        assert_eq!(pattern_data.tracks.len(), 1);
        assert_eq!(pattern_data.bass_tracks.len(), 2);

        // Test bass pattern conversion
        let bass_patterns = pattern_data.to_bass_patterns().expect("Should convert bass patterns");
        assert_eq!(bass_patterns.len(), 2);

        // Test that patterns can generate events (requires dummy chord progression)
        let chords = vec![
            crate::Chord::major(crate::PitchClass::C),
            crate::Chord::major(crate::PitchClass::F),
        ];

        for pattern in bass_patterns {
            let events = pattern.generate(4, &chords);
            assert!(!events.is_empty(), "Bass pattern should generate events");
        }
    }
}
