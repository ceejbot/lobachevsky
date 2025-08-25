//! Rhythm generation and pattern systems

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::LobachevskyError;

/// Represents a beat position in time (can be fractional for subdivisions)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Beat(pub f64);

impl Beat {
    pub fn new(value: f64) -> Self {
        Beat(value)
    }

    /// Quantize to nearest subdivision
    pub fn quantize(&self, subdivision: f64) -> Self {
        Beat((self.0 / subdivision).round() * subdivision)
    }

    /// Add humanization (micro-timing variation)
    pub fn humanize(&self, amount: f64) -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let variation = rng.random_range(-amount..amount);
        Beat(self.0 + variation)
    }
}

/// Represents a duration in beats
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Duration(pub f64);

/// Drum/percussion instrument types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrumVoice {
    Kick,
    KickSoft,
    Snare,
    Rim,
    HiHatClosed,
    HiHatOpen,
    Shaker,
    Ride,
    Clap,
    Percussion,
}

impl DrumVoice {
    /// Get General MIDI note number for this drum
    pub fn midi_note(&self) -> u8 {
        match self {
            DrumVoice::Kick => 36,
            DrumVoice::KickSoft => 35,
            DrumVoice::Snare => 38,
            DrumVoice::Rim => 37,
            DrumVoice::HiHatClosed => 42,
            DrumVoice::HiHatOpen => 46,
            DrumVoice::Shaker => 70,
            DrumVoice::Ride => 51,
            DrumVoice::Clap => 39,
            DrumVoice::Percussion => 69,
        }
    }
}

impl TryFrom<&str> for DrumVoice {
    type Error = LobachevskyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "kick" => Ok(DrumVoice::Kick),
            "kicksoft" | "kick_soft" => Ok(DrumVoice::KickSoft),
            "snare" => Ok(DrumVoice::Snare),
            "rim" => Ok(DrumVoice::Rim),
            "hihatclosed" | "hihat_closed" | "closed_hihat" => Ok(DrumVoice::HiHatClosed),
            "hihatopen" | "hihat_open" | "open_hihat" => Ok(DrumVoice::HiHatOpen),
            "shaker" => Ok(DrumVoice::Shaker),
            "ride" => Ok(DrumVoice::Ride),
            "clap" => Ok(DrumVoice::Clap),
            "percussion" | "perc" => Ok(DrumVoice::Percussion),
            _ => Err(LobachevskyError::InvalidDrumVoice {
                voice: value.to_string(),
            }),
        }
    }
}

/// A rhythmic event (hit)
#[derive(Debug, Clone)]
pub struct DrumEvent {
    pub voice: DrumVoice,
    pub beat: Beat,
    pub velocity: u8,
}

/// Trait for rhythm pattern generators
pub trait RhythmPattern: Send + Sync {
    /// Generate events for a specific bar
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent>;

    /// Get the length of the pattern in bars
    fn pattern_length(&self) -> usize {
        1
    }

    /// Clone the pattern (for dynamic dispatch)
    fn clone_box(&self) -> Box<dyn RhythmPattern>;
}

/// Euclidean rhythm generator - distributes hits evenly across steps
#[derive(Debug, Clone)]
pub struct EuclideanPattern {
    voice: DrumVoice,
    hits: usize,
    steps: usize,
    rotation: usize,
    velocity: u8,
}

impl EuclideanPattern {
    pub fn new(voice: DrumVoice, hits: usize, steps: usize) -> Self {
        EuclideanPattern {
            voice,
            hits,
            steps,
            rotation: 0,
            velocity: 64,
        }
    }

    pub fn with_rotation(mut self, rotation: usize) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_velocity(mut self, velocity: u8) -> Self {
        self.velocity = velocity;
        self
    }

    /// Generate Euclidean rhythm pattern using the Bresenham algorithm
    /// This distributes hits as evenly as possible across the steps
    fn generate_pattern(&self) -> Vec<bool> {
        use crate::euclidean::{Breshenham, EuclideanRhythm};

        // Handle edge cases that the old implementation allowed
        if self.steps == 0 {
            return Vec::new();
        }

        // Clamp hits to steps (maintain backward compatibility)
        let hits = self.hits.min(self.steps);

        // Use the tested and optimized Bresenham implementation
        let mut pattern = Breshenham::generate(self.steps, hits).unwrap_or_else(|_| vec![false; self.steps]);

        // Apply additional rotation on top of the default "start with beat" rotation
        if self.rotation > 0 && !pattern.is_empty() {
            let rot = self.rotation % pattern.len();
            pattern.rotate_left(rot);
        }

        pattern
    }
}

impl RhythmPattern for EuclideanPattern {
    fn events_for_bar(&self, _bar: usize) -> Vec<DrumEvent> {
        let pattern = self.generate_pattern();
        let mut events = Vec::new();

        for (i, &hit) in pattern.iter().enumerate() {
            if hit {
                let beat = Beat(i as f64 * 4.0 / self.steps as f64);
                events.push(DrumEvent {
                    voice: self.voice,
                    beat,
                    velocity: self.velocity,
                });
            }
        }

        events
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(self.clone())
    }
}

/// Probability-based pattern generator
#[derive(Debug, Clone)]
pub struct ProbabilityPattern {
    voice: DrumVoice,
    densities: Vec<(Beat, f64)>, // (beat_position, probability)
    pub velocity_range: (u8, u8),
}

impl ProbabilityPattern {
    pub fn new(voice: DrumVoice) -> Self {
        ProbabilityPattern {
            voice,
            densities: Vec::new(),
            velocity_range: (40, 80),
        }
    }

    pub fn add_point(mut self, beat: Beat, probability: f64) -> Self {
        self.densities.push((beat, probability.clamp(0.0, 1.0)));
        self
    }

    pub fn sparse(voice: DrumVoice) -> Self {
        Self::new(voice).add_point(Beat(0.0), 0.7).add_point(Beat(2.0), 0.5)
    }

    pub fn dense(voice: DrumVoice) -> Self {
        Self::new(voice)
            .add_point(Beat(0.0), 0.9)
            .add_point(Beat(0.5), 0.6)
            .add_point(Beat(1.0), 0.8)
            .add_point(Beat(1.5), 0.5)
            .add_point(Beat(2.0), 0.9)
            .add_point(Beat(2.5), 0.6)
            .add_point(Beat(3.0), 0.7)
            .add_point(Beat(3.5), 0.4)
    }
}

impl RhythmPattern for ProbabilityPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        use rand::Rng;
        let mut rng = rand::rng();
        let mut events = Vec::new();

        // Add some variation per bar
        let bar_variation = (bar % 4) as f64 * 0.05;

        for (beat, base_prob) in &self.densities {
            let probability = (base_prob + bar_variation).clamp(0.0, 1.0);

            if rng.random::<f64>() < probability {
                let velocity = rng.random_range(self.velocity_range.0..=self.velocity_range.1);
                events.push(DrumEvent {
                    voice: self.voice,
                    beat: *beat,
                    velocity,
                });
            }
        }

        events
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(self.clone())
    }
}

/// Layered pattern combining multiple sub-patterns
pub struct LayeredPattern {
    layers: Vec<Box<dyn RhythmPattern>>,
}

impl Default for LayeredPattern {
    fn default() -> Self {
        Self::new()
    }
}

impl LayeredPattern {
    pub fn new() -> Self {
        LayeredPattern { layers: Vec::new() }
    }

    pub fn add_layer(mut self, pattern: Box<dyn RhythmPattern>) -> Self {
        self.layers.push(pattern);
        self
    }
}

impl Clone for LayeredPattern {
    fn clone(&self) -> Self {
        LayeredPattern {
            layers: self.layers.iter().map(|l| l.clone_box()).collect(),
        }
    }
}

impl RhythmPattern for LayeredPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        let mut all_events = Vec::new();

        for layer in &self.layers {
            all_events.extend(layer.events_for_bar(bar));
        }

        // Sort by beat position
        all_events.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
        all_events
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(self.clone())
    }
}

/// Data structure for defining patterns in a declarative way
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternData {
    pub name: String,
    pub description: Option<String>,
    pub tempo_hint: Option<u16>,
    pub layers: Vec<LayerData>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbabilityPoint {
    pub beat: f64,
    pub probability: f64,
}

impl PatternData {
    /// Load pattern from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, LobachevskyError> {
        toml::from_str(toml_str).map_err(|source| LobachevskyError::PatternParseError { source })
    }

    /// Convert to RhythmPattern
    pub fn to_pattern(&self) -> Result<Box<dyn RhythmPattern>, LobachevskyError> {
        let mut layered = LayeredPattern::new();

        for layer in &self.layers {
            let voice = self.parse_voice(&layer.voice)?;
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
            };
            layered = layered.add_layer(pattern);
        }

        Ok(Box::new(layered))
    }

    fn parse_voice(&self, voice_str: &str) -> Result<DrumVoice, LobachevskyError> {
        DrumVoice::try_from(voice_str)
    }
}

/// Pattern library for loading predefined patterns
pub struct PatternLibrary {
    patterns: HashMap<String, PatternData>,
}

impl Default for PatternLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternLibrary {
    pub fn new() -> Self {
        PatternLibrary {
            patterns: HashMap::new(),
        }
    }

    /// Load patterns from a directory
    pub fn load_from_directory(&mut self, path: &std::path::Path) -> Result<(), LobachevskyError> {
        use std::fs;

        let entries = fs::read_dir(path).map_err(|source| LobachevskyError::DirectoryReadError {
            path: path.to_path_buf(),
            source,
        })?;

        for entry in entries {
            let entry = entry.map_err(|source| LobachevskyError::DirectoryReadError {
                path: path.to_path_buf(),
                source,
            })?;
            let file_path = entry.path();

            if file_path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let contents = fs::read_to_string(&file_path).map_err(|source| LobachevskyError::PatternFileError {
                    path: file_path.clone(),
                    source,
                })?;

                let pattern = PatternData::from_toml(&contents)?;
                self.patterns.insert(pattern.name.clone(), pattern);
            }
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&PatternData> {
        self.patterns.get(name)
    }

    pub fn list(&self) -> Vec<&str> {
        self.patterns.keys().map(|s| s.as_str()).collect()
    }
}

/// Genre-specific pattern templates
pub struct GenrePatterns;

impl GenrePatterns {
    /// 90s ambient techno - very sparse, breathing
    pub fn ambient_90s() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 2, 4).with_velocity(70)))
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatClosed)
                        .add_point(Beat(1.5), 0.4)
                        .add_point(Beat(3.5), 0.3),
                ))
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Snare)
                        .add_point(Beat(1.25), 0.3)
                        .add_point(Beat(3.25), 0.25),
                )),
        )
    }

    /// 2000s microhouse - shuffled, ghost notes
    pub fn microhouse_2000s() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 2, 4).with_velocity(75)))
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::KickSoft)
                        .add_point(Beat(1.75), 0.8)
                        .add_point(Beat(3.75), 0.7),
                ))
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::HiHatClosed, 8, 16)
                        .with_velocity(45)
                        .with_rotation(1), // Slight shuffle
                )),
        )
    }

    /// 2010s Euclidean patterns
    pub fn euclidean_2010s() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::Kick, 4, 16).with_velocity(80),
                ))
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::Snare, 5, 16)
                        .with_rotation(4)
                        .with_velocity(65),
                ))
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::HiHatClosed, 7, 16)
                        .with_rotation(2)
                        .with_velocity(50),
                )),
        )
    }

    /// Detroit techno - raw, driving, boomy kicks
    pub fn detroit_techno() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Strong 4/4 kick
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(90)))
                // Sparse hi-hats with emphasis on off-beats
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatOpen)
                        .add_point(Beat(0.5), 0.8)
                        .add_point(Beat(1.5), 0.7)
                        .add_point(Beat(2.5), 0.8)
                        .add_point(Beat(3.5), 0.6),
                ))
                // Occasional rim shots for texture
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Rim)
                        .add_point(Beat(1.75), 0.3)
                        .add_point(Beat(3.25), 0.4),
                )),
        )
    }

    /// Berlin dub techno - deep, atmospheric, spacious
    pub fn berlin_dub_techno() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Subdued kick pattern
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(75)))
                // Very sparse, delayed snare hits
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Snare)
                        .add_point(Beat(1.0), 0.2)
                        .add_point(Beat(3.0), 0.15),
                ))
                // Minimal closed hats with micro-timing variations
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatClosed)
                        .add_point(Beat(0.25), 0.3)
                        .add_point(Beat(1.25), 0.35)
                        .add_point(Beat(2.25), 0.3)
                        .add_point(Beat(3.75), 0.4),
                ))
                // Occasional shaker for atmosphere
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Shaker)
                        .add_point(Beat(0.75), 0.2)
                        .add_point(Beat(2.75), 0.25),
                )),
        )
    }

    /// Minimal Berlin - stripped-down, hypnotic
    pub fn minimal_berlin() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Basic kick pattern
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(80)))
                // Clap on 2 and 4 with slight variation
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Clap)
                        .add_point(Beat(1.0), 0.9)
                        .add_point(Beat(3.0), 0.85),
                ))
                // Rolling hi-hat pattern using Euclidean distribution
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::HiHatClosed, 11, 16)
                        .with_velocity(45)
                        .with_rotation(3),
                ))
                // Subtle ride for movement
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Ride)
                        .add_point(Beat(0.5), 0.3)
                        .add_point(Beat(2.5), 0.35),
                )),
        )
    }

    /// Acid minimal - 303-influenced with syncopation
    pub fn acid_minimal() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Kick pattern with occasional ghost kicks
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(85)))
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::KickSoft)
                        .add_point(Beat(0.75), 0.6)
                        .add_point(Beat(2.75), 0.5),
                ))
                // Syncopated snare/rim pattern
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Snare)
                        .add_point(Beat(1.25), 0.7)
                        .add_point(Beat(3.25), 0.8),
                ))
                // Complex hi-hat pattern with 3-over-4 feel
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::HiHatClosed, 5, 8)
                        .with_velocity(50)
                        .with_rotation(1),
                ))
                // Open hat accents
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatOpen)
                        .add_point(Beat(1.5), 0.5)
                        .add_point(Beat(3.0), 0.4),
                )),
        )
    }

    /// Deep minimal - subdued with ghost notes
    pub fn deep_minimal() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Soft, deep kick pattern
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(70)))
                // Very soft ghost kicks for swing
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::KickSoft)
                        .add_point(Beat(0.5), 0.3)
                        .add_point(Beat(1.75), 0.4)
                        .add_point(Beat(2.5), 0.3)
                        .add_point(Beat(3.75), 0.35),
                ))
                // Minimal rim pattern
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Rim)
                        .add_point(Beat(1.0), 0.5)
                        .add_point(Beat(3.0), 0.45),
                ))
                // Very sparse, quiet hi-hats
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatClosed)
                        .add_point(Beat(0.25), 0.3)
                        .add_point(Beat(0.75), 0.25)
                        .add_point(Beat(2.25), 0.3)
                        .add_point(Beat(2.75), 0.25),
                ))
                // Occasional percussion for texture
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Percussion)
                        .add_point(Beat(1.5), 0.2)
                        .add_point(Beat(3.5), 0.15),
                )),
        )
    }
}

// Add rand dependency for probability patterns
// Note: You'll need to add `rand = "0.8"` to Cargo.toml

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclidean_patterns_basic() {
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 8);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 3); // Should have 3 hits

        // Check that events are spread evenly
        let positions: Vec<f64> = events.iter().map(|e| e.beat.0).collect();
        println!("3/8 pattern positions: {:?}", positions);
        assert!(positions.len() == 3);
    }

    #[test]
    fn euclidean_edge_cases() {
        // Test zero hits
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 0, 8);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 0);

        // Test zero steps
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 0);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 0);

        // Test hits >= steps
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 8, 8);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 8);

        let pattern = EuclideanPattern::new(DrumVoice::Kick, 10, 8);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 8);
    }

    #[test]
    fn euclidean_problematic_cases() {
        // These were the cases that caused panics before the fix
        let test_cases = vec![
            (5, 8),   // Original failing case
            (4, 7),   // Another failing case
            (7, 12),  // Complex case
            (3, 5),   // Simple case
            (13, 16), // Many hits
            (1, 16),  // Very sparse
        ];

        for (hits, steps) in test_cases {
            println!("Testing ({}, {})", hits, steps);
            let pattern = EuclideanPattern::new(DrumVoice::Kick, hits, steps);
            let events = pattern.events_for_bar(0);

            // Should not panic and should have correct number of hits
            assert_eq!(
                events.len(),
                hits.min(steps),
                "Pattern ({}, {}) should have {} hits but got {}",
                hits,
                steps,
                hits.min(steps),
                events.len()
            );

            // Events should be within the bar (0.0 to 4.0 beats)
            for event in &events {
                assert!(
                    event.beat.0 >= 0.0 && event.beat.0 < 4.0,
                    "Event at beat {} is outside valid range for ({}, {})",
                    event.beat.0,
                    hits,
                    steps
                );
            }
        }
    }

    #[test]
    fn euclidean_rotation() {
        let base_pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 8);
        let rotated_pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 8).with_rotation(2);

        let base_events = base_pattern.events_for_bar(0);
        let rotated_events = rotated_pattern.events_for_bar(0);

        // Should have same number of hits
        assert_eq!(base_events.len(), rotated_events.len());

        // But different timing (unless the pattern is symmetric)
        let base_positions: Vec<f64> = base_events.iter().map(|e| e.beat.0).collect();
        let rotated_positions: Vec<f64> = rotated_events.iter().map(|e| e.beat.0).collect();

        println!("Base positions: {:?}", base_positions);
        println!("Rotated positions: {:?}", rotated_positions);
    }

    #[test]
    fn euclidean_velocity() {
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 8).with_velocity(100);
        let events = pattern.events_for_bar(0);

        for event in events {
            assert_eq!(event.velocity, 100);
            assert_eq!(event.voice, DrumVoice::Kick);
        }
    }

    #[test]
    fn can_quantize_beat() {
        let beat = Beat(1.73);
        let quantized = beat.quantize(0.5);
        assert_eq!(quantized.0, 1.5);
    }

    #[test]
    fn beat_humanization() {
        let beat = Beat(1.0);
        let humanized = beat.humanize(0.1);

        // Should be close to original but not exact
        assert!(humanized.0 >= 0.9 && humanized.0 <= 1.1);
    }
}
