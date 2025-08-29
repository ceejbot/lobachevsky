//! Terry Riley's "In C" pattern representation and parsing

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{LobachevskyError, Note, PitchClass};

/// Represents a single note or rest in a Riley pattern
#[derive(Debug, Clone, PartialEq)]
pub struct RileyNote {
    /// The pitch (None for rests)
    pub pitch: Option<Note>,
    /// Duration as a fraction (e.g., 0.25 for quarter note, 0.125 for eighth)
    pub duration: f32,
    /// Whether this note is tied to the next
    pub tied: bool,
}

impl RileyNote {
    /// Create a new note
    pub fn new(pitch: Option<Note>, duration: f32) -> Self {
        Self {
            pitch,
            duration,
            tied: false,
        }
    }

    /// Create a tied note
    pub fn new_tied(pitch: Option<Note>, duration: f32) -> Self {
        Self {
            pitch,
            duration,
            tied: true,
        }
    }

    /// Create a rest
    pub fn rest(duration: f32) -> Self {
        Self {
            pitch: None,
            duration,
            tied: false,
        }
    }

    /// Parse from notation string (e.g., "C4/8", "rest/4", "G4/16~")
    pub fn parse(notation: &str) -> Result<Self, LobachevskyError> {
        let notation = notation.trim();

        // Check for tie marker
        let tied = notation.ends_with('~');
        let notation = if tied {
            &notation[..notation.len() - 1]
        } else {
            notation
        };

        // Split into pitch and duration
        let parts: Vec<&str> = notation.split('/').collect();
        if parts.len() != 2 {
            return Err(LobachevskyError::ParseError {
                message: format!("Invalid note notation: {}", notation),
            });
        }

        let pitch_str = parts[0];
        let duration_str = parts[1];

        // Parse pitch
        let pitch = if pitch_str.to_lowercase() == "rest" {
            None
        } else {
            Some(Self::parse_note(pitch_str)?)
        };

        // Parse duration (with support for dots)
        let duration = Self::parse_duration(duration_str)?;

        Ok(Self { pitch, duration, tied })
    }

    /// Parse a note string (e.g., "C4", "E4", "F#4")
    fn parse_note(note_str: &str) -> Result<Note, LobachevskyError> {
        // Split into pitch class and octave
        let (pitch_str, octave_str) = if note_str.contains('#') {
            // Handle sharps
            let parts: Vec<&str> = note_str.splitn(2, '#').collect();
            if parts.len() != 2 {
                return Err(LobachevskyError::ParseError {
                    message: format!("Invalid note: {}", note_str),
                });
            }
            (format!("{}#", parts[0]), parts[1].to_string())
        } else if note_str.contains('b') {
            // Handle flats
            let parts: Vec<&str> = note_str.splitn(2, 'b').collect();
            if parts.len() != 2 {
                return Err(LobachevskyError::ParseError {
                    message: format!("Invalid note: {}", note_str),
                });
            }
            (format!("{}b", parts[0]), parts[1].to_string())
        } else {
            // Natural note
            let pitch_char = &note_str[0..1];
            let octave_part = &note_str[1..];
            (pitch_char.to_string(), octave_part.to_string())
        };

        let pitch_class = PitchClass::try_from(pitch_str.as_str())?;
        let octave: i8 = octave_str.parse().map_err(|_| LobachevskyError::ParseError {
            message: format!("Invalid octave: {}", octave_str),
        })?;

        Ok(Note::new(pitch_class, octave))
    }

    /// Parse duration string (e.g., "4", "8", "4.", "2.")
    fn parse_duration(duration_str: &str) -> Result<f32, LobachevskyError> {
        let dotted = duration_str.ends_with('.');
        let base_str = if dotted {
            &duration_str[..duration_str.len() - 1]
        } else {
            duration_str
        };

        let denominator: f32 = base_str.parse().map_err(|_| LobachevskyError::ParseError {
            message: format!("Invalid duration: {}", duration_str),
        })?;

        let mut duration = 1.0 / denominator;
        if dotted {
            duration *= 1.5;
        }

        Ok(duration)
    }

    /// Apply a tempo ratio transformation (e.g., 2.0 for double-time)
    pub fn with_tempo_ratio(&self, ratio: f32) -> Self {
        Self {
            pitch: self.pitch,
            duration: self.duration / ratio,
            tied: self.tied,
        }
    }
}

/// Represents one of the 53 patterns in "In C"
#[derive(Debug, Clone)]
pub struct RileyPattern {
    /// Pattern number (1-53)
    pub number: usize,
    /// The notes in this pattern
    pub notes: Vec<RileyNote>,
    /// Description of the pattern
    pub description: Option<String>,
    /// Whether this pattern works well in canon with itself
    pub canon_compatible: bool,
    /// Suggested beat offsets for canon voices
    pub canon_offsets: Vec<f32>,
    /// Whether tempo variations work well with this pattern
    pub tempo_variation_suitable: bool,
    /// How well the pattern harmonizes with itself
    pub self_harmony: SelfHarmony,
}

/// How well a pattern harmonizes with itself when played in canon
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SelfHarmony {
    /// Creates consonant harmonies (thirds, fifths, octaves)
    Consonant,
    /// Creates mild dissonances that resolve
    Moderate,
    /// Creates strong dissonances or clashes
    Dissonant,
    /// Single notes or unisons only
    Neutral,
}

impl fmt::Display for SelfHarmony {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelfHarmony::Consonant => write!(f, "consonant"),
            SelfHarmony::Moderate => write!(f, "moderate"),
            SelfHarmony::Dissonant => write!(f, "dissonant"),
            SelfHarmony::Neutral => write!(f, "neutral"),
        }
    }
}

impl From<&str> for SelfHarmony {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "consonant" => SelfHarmony::Consonant,
            "moderate" => SelfHarmony::Moderate,
            "dissonant" => SelfHarmony::Dissonant,
            _ => SelfHarmony::Neutral,
        }
    }
}

impl RileyPattern {
    /// Parse a pattern from notation string
    pub fn parse(number: usize, notation: &str) -> Result<Self, LobachevskyError> {
        let note_strings: Vec<&str> = notation.split_whitespace().collect();
        let mut notes = Vec::new();

        for note_str in note_strings {
            notes.push(RileyNote::parse(note_str)?);
        }

        Ok(Self {
            number,
            notes,
            description: None,
            canon_compatible: false,
            canon_offsets: vec![],
            tempo_variation_suitable: false,
            self_harmony: SelfHarmony::Neutral,
        })
    }

    /// Get the total duration of this pattern in beats
    pub fn duration(&self) -> f32 {
        self.notes.iter().map(|n| n.duration).sum()
    }

    /// Apply a tempo ratio to all notes
    pub fn with_tempo_ratio(&self, ratio: f32) -> Self {
        Self {
            number: self.number,
            notes: self.notes.iter().map(|n| n.with_tempo_ratio(ratio)).collect(),
            description: self.description.clone(),
            canon_compatible: self.canon_compatible,
            canon_offsets: self.canon_offsets.clone(),
            tempo_variation_suitable: self.tempo_variation_suitable,
            self_harmony: self.self_harmony,
        }
    }

    /// Check if this pattern has sustained notes suitable for canon
    pub fn has_sustained_notes(&self) -> bool {
        self.notes.iter().any(|n| n.duration >= 0.5) // Half note or longer
    }

    /// Check if this pattern is primarily rhythmic (good for tempo variations)
    pub fn is_rhythmic(&self) -> bool {
        let short_notes = self.notes.iter().filter(|n| n.duration <= 0.25).count();
        short_notes as f32 / self.notes.len() as f32 > 0.6
    }

    /// Analyze pattern for automatic canon compatibility
    pub fn analyze_canon_compatibility(&mut self) {
        // Patterns with sustained notes work well in canon
        if self.has_sustained_notes() {
            self.canon_compatible = true;

            // Suggest offsets based on pattern duration
            let dur = self.duration();
            if dur >= 2.0 {
                self.canon_offsets = vec![1.0, 2.0];
            } else if dur >= 1.0 {
                self.canon_offsets = vec![0.5, 1.0];
            } else {
                self.canon_offsets = vec![0.25, 0.5];
            }
        }

        // Rhythmic patterns are good for tempo variations
        self.tempo_variation_suitable = self.is_rhythmic();

        // Analyze self-harmony based on intervals
        self.analyze_self_harmony();
    }

    /// Analyze how well this pattern harmonizes with itself
    fn analyze_self_harmony(&mut self) {
        let pitches: Vec<Note> = self.notes.iter().filter_map(|n| n.pitch).collect();

        if pitches.is_empty() {
            self.self_harmony = SelfHarmony::Neutral;
            return;
        }

        // Check for repeated notes (neutral)
        let mut unique_pitches = Vec::new();
        for pitch in &pitches {
            if !unique_pitches.iter().any(|p: &Note| p == pitch) {
                unique_pitches.push(*pitch);
            }
        }
        if unique_pitches.len() == 1 {
            self.self_harmony = SelfHarmony::Neutral;
            return;
        }

        // Analyze intervals between pitches
        let mut consonant = 0;
        let mut dissonant = 0;

        for i in 0..pitches.len() {
            for j in i + 1..pitches.len() {
                let interval = (pitches[j].to_midi() as i32 - pitches[i].to_midi() as i32).abs() % 12;
                match interval {
                    0 | 3 | 4 | 7 | 8 | 9 => consonant += 1, // Unison, m3, M3, P5, m6, M6
                    1 | 2 | 6 | 10 | 11 => dissonant += 1,   // m2, M2, tritone, m7, M7
                    _ => {}
                }
            }
        }

        self.self_harmony = if consonant > dissonant * 2 {
            SelfHarmony::Consonant
        } else if dissonant > consonant * 2 {
            SelfHarmony::Dissonant
        } else {
            SelfHarmony::Moderate
        };
    }
}

/// TOML representation of a Riley pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RileyPatternData {
    pub number: usize,
    pub notes: String,
    pub description: Option<String>,
    #[serde(default)]
    pub canon_compatible: bool,
    #[serde(default)]
    pub canon_offsets: Vec<f32>,
    #[serde(default)]
    pub tempo_variation_suitable: bool,
    #[serde(default = "default_self_harmony")]
    pub self_harmony: String,
}

fn default_self_harmony() -> String {
    "neutral".to_string()
}

impl RileyPatternData {
    /// Convert to internal RileyPattern
    pub fn to_pattern(&self) -> Result<RileyPattern, LobachevskyError> {
        let mut pattern = RileyPattern::parse(self.number, &self.notes)?;
        pattern.description = self.description.clone();
        pattern.canon_compatible = self.canon_compatible;
        pattern.canon_offsets = self.canon_offsets.clone();
        pattern.tempo_variation_suitable = self.tempo_variation_suitable;
        pattern.self_harmony = SelfHarmony::from(self.self_harmony.as_str());

        // If no manual analysis was provided, auto-analyze
        if !self.canon_compatible && self.canon_offsets.is_empty() {
            pattern.analyze_canon_compatibility();
        }

        Ok(pattern)
    }
}

/// Collection of all 53 patterns
#[derive(Debug, Clone)]
pub struct InCPatterns {
    pub patterns: Vec<RileyPattern>,
}

impl InCPatterns {
    /// Load patterns from TOML file
    pub fn from_toml(toml_str: &str) -> Result<Self, LobachevskyError> {
        #[derive(Deserialize)]
        struct PatternFile {
            patterns: Vec<RileyPatternData>,
        }

        let file: PatternFile = toml::from_str(toml_str).map_err(|e| LobachevskyError::ParseError {
            message: format!("Failed to parse pattern file: {}", e),
        })?;

        let mut patterns = Vec::new();
        for data in file.patterns {
            patterns.push(data.to_pattern()?);
        }

        Ok(Self { patterns })
    }

    /// Get a pattern by number (1-53)
    pub fn get(&self, number: usize) -> Option<&RileyPattern> {
        self.patterns.iter().find(|p| p.number == number)
    }

    /// Get patterns that work well in canon
    pub fn canon_compatible_patterns(&self) -> Vec<&RileyPattern> {
        self.patterns.iter().filter(|p| p.canon_compatible).collect()
    }

    /// Get patterns suitable for tempo variations
    pub fn tempo_variation_patterns(&self) -> Vec<&RileyPattern> {
        self.patterns.iter().filter(|p| p.tempo_variation_suitable).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note() {
        let note = RileyNote::parse("C4/8").expect("should parse music notation");
        assert_eq!(note.duration, 0.125);
        assert!(!note.tied);

        let rest = RileyNote::parse("rest/4").expect("should parse music notation");
        assert_eq!(rest.pitch, None);
        assert_eq!(rest.duration, 0.25);

        let tied = RileyNote::parse("G4/16~").expect("should parse music notation");
        assert!(tied.tied);
        assert_eq!(tied.duration, 0.0625);

        let dotted = RileyNote::parse("E4/4.").expect("should parse music notation");
        assert_eq!(dotted.duration, 0.375); // 0.25 * 1.5
    }

    #[test]
    fn test_tempo_ratio() {
        let note = RileyNote::parse("C4/8").expect("should parse music notation");
        let double_time = note.with_tempo_ratio(2.0);
        assert_eq!(double_time.duration, 0.0625); // Half the duration

        let half_time = note.with_tempo_ratio(0.5);
        assert_eq!(half_time.duration, 0.25); // Double the duration
    }

    #[test]
    fn test_pattern_duration() {
        let pattern = RileyPattern::parse(1, "C4/8 E4/8 C4/8 E4/8").expect("should parse music notation");
        assert_eq!(pattern.duration(), 0.5); // 4 eighth notes = half a measure
    }
}
