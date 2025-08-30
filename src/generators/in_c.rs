//! Terry Riley's "In C" algorithmic performance generator

use rand::Rng;

use crate::melody::InCPatterns;
use crate::midi::MidiFile;
use crate::{LobachevskyError, Note};

/// Represents a voice playing a pattern at an offset
#[derive(Debug, Clone)]
pub struct CanonVoice {
    /// Offset in beats from the main voice
    pub offset_beats: f32,
    /// Velocity ratio (0.0-1.0) for dynamics
    pub velocity_ratio: f32,
    /// Pan position (-1.0 to 1.0)
    pub pan_position: f32,
}

/// Types of timing variations a performer can apply
#[derive(Debug, Clone, Copy)]
pub enum TimingVariation {
    /// Normal tempo
    Normal,
    /// Double-time (2x speed)
    DoubleTime,
    /// Half-time (0.5x speed)
    HalfTime,
    /// Custom ratio
    Custom(f32),
}

impl TimingVariation {
    /// Get the tempo ratio multiplier
    pub fn ratio(&self) -> f32 {
        match self {
            TimingVariation::Normal => 1.0,
            TimingVariation::DoubleTime => 2.0,
            TimingVariation::HalfTime => 0.5,
            TimingVariation::Custom(r) => *r,
        }
    }

    /// Choose a random variation based on probability
    pub fn random_variation(allow_variations: bool) -> Self {
        if !allow_variations {
            return TimingVariation::Normal;
        }

        let mut rng = rand::rng();
        let choice: f32 = rng.random_range(0.0..1.0);

        if choice < 0.7 {
            TimingVariation::Normal
        } else if choice < 0.85 {
            TimingVariation::DoubleTime
        } else if choice < 0.95 {
            TimingVariation::HalfTime
        } else {
            // Occasionally use other ratios
            let ratios = [0.75, 1.5, 0.66, 1.33];
            TimingVariation::Custom(ratios[rng.random_range(0..ratios.len())])
        }
    }
}

/// Represents a single performer in the ensemble
#[derive(Debug, Clone)]
pub struct InCPerformer {
    /// Unique ID for this performer
    pub id: usize,
    /// Current pattern number (1-53)
    pub current_pattern: usize,
    /// How many more times to repeat the current pattern
    pub repetitions_remaining: usize,
    /// Current timing variation
    pub timing_variation: TimingVariation,
    /// Canon voices (if playing pattern against itself)
    pub canon_voices: Vec<CanonVoice>,
    /// Whether currently listening (not playing)
    pub is_listening: bool,
    /// Position in the current pattern (note index)
    pub pattern_position: usize,
    /// Time until next note (in beats)
    pub time_to_next_note: f32,
    /// MIDI channel for this performer
    pub midi_channel: u8,
    /// Base velocity for this performer
    pub base_velocity: u8,
    /// Octave transposition (-2 to +2)
    pub octave_transpose: i8,
}

impl InCPerformer {
    /// Create a new performer
    pub fn new(id: usize, midi_channel: u8) -> Self {
        let mut rng = rand::rng();

        Self {
            id,
            current_pattern: 1,
            repetitions_remaining: rng.random_range(4..12),
            timing_variation: TimingVariation::Normal,
            canon_voices: vec![],
            is_listening: false,
            pattern_position: 0,
            time_to_next_note: 0.0,
            midi_channel,
            base_velocity: rng.random_range(60..90),
            octave_transpose: 0,
        }
    }

    /// Decide whether to advance to the next pattern
    pub fn maybe_advance_pattern(&mut self, ensemble_median: usize, patterns: &InCPatterns) {
        // Check if we're too far ahead or behind
        let distance = (self.current_pattern as i32 - ensemble_median as i32).abs();

        let mut rng = rand::rng();

        // More likely to advance if we're behind
        let advance_probability = if self.current_pattern < ensemble_median {
            0.8
        } else if distance > 2 {
            0.3 // Less likely if we're far ahead
        } else {
            0.6
        };

        if rng.random_range(0.0..1.0) < advance_probability && self.current_pattern < 53 {
            self.current_pattern += 1;
            self.pattern_position = 0;
            self.time_to_next_note = 0.0;

            // Decide repetitions for new pattern
            self.repetitions_remaining = rng.random_range(3..10);

            // Maybe change timing variation
            if rng.random_range(0.0..1.0) < 0.2
                && let Some(pattern) = patterns.get(self.current_pattern)
            {
                self.timing_variation = if pattern.tempo_variation_suitable {
                    TimingVariation::random_variation(true)
                } else {
                    TimingVariation::Normal
                };
            }

            // Maybe add canon voices
            self.maybe_add_canon(patterns);

            // Maybe transpose octave
            if rng.random_range(0.0..1.0) < 0.1 {
                self.octave_transpose = rng.random_range(-1..=1);
            }
        }
    }

    /// Decide whether to add canon voices for the current pattern
    fn maybe_add_canon(&mut self, patterns: &InCPatterns) {
        let mut rng = rand::rng();

        if let Some(pattern) = patterns.get(self.current_pattern) {
            if pattern.canon_compatible && rng.random_range(0.0..1.0) < 0.3 {
                self.canon_voices.clear();

                // Add 1-2 canon voices
                let num_voices = rng.random_range(1..=2);

                for i in 0..num_voices {
                    let offset = if !pattern.canon_offsets.is_empty() {
                        pattern.canon_offsets[i % pattern.canon_offsets.len()]
                    } else {
                        (i + 1) as f32 * 0.5
                    };

                    self.canon_voices.push(CanonVoice {
                        offset_beats: offset,
                        velocity_ratio: 0.6 + (i as f32 * 0.1),
                        pan_position: if i == 0 { -0.3 } else { 0.3 },
                    });
                }
            } else {
                self.canon_voices.clear();
            }
        }
    }

    /// Decide whether to start listening (drop out)
    pub fn maybe_start_listening(&mut self) {
        let mut rng = rand::rng();

        if !self.is_listening && rng.random_range(0.0..1.0) < 0.05 {
            self.is_listening = true;
        } else if self.is_listening && rng.random_range(0.0..1.0) < 0.2 {
            self.is_listening = false;
        }
    }
}

/// Configuration for In C performance
#[derive(Debug, Clone)]
pub struct InCConfig {
    /// Number of performers
    pub num_performers: usize,
    /// Performance duration in minutes
    pub duration_minutes: f32,
    /// Include eighth-note pulse
    pub include_pulse: bool,
    /// Tempo in BPM
    pub tempo: u16,
    /// Performance variation (0.0-1.0)
    pub variation: f32,
    /// Probability of canon voices
    pub canon_probability: f32,
    /// Amount of timing flexibility
    pub timing_flex: f32,
}

impl Default for InCConfig {
    fn default() -> Self {
        Self {
            num_performers: 12,
            duration_minutes: 20.0,
            include_pulse: true,
            tempo: 120,
            variation: 0.5,
            canon_probability: 0.3,
            timing_flex: 0.3,
        }
    }
}

/// Generator for In C performances
#[derive(Debug, Clone)]
pub struct InCGenerator {
    config: InCConfig,
    patterns: InCPatterns,
    performers: Vec<InCPerformer>,
}

impl InCGenerator {
    /// Create a new In C generator
    pub fn new(config: InCConfig, patterns: InCPatterns) -> Self {
        let mut performers = Vec::new();

        for i in 0..config.num_performers {
            let channel = (i % 16) as u8;
            performers.push(InCPerformer::new(i, channel));
        }

        Self {
            config,
            patterns,
            performers,
        }
    }

    /// Generate a complete In C performance
    pub fn generate(&mut self) -> Result<MidiFile, LobachevskyError> {
        let mut midi_file = MidiFile::new(self.config.tempo);

        // Add pulse track if requested
        if self.config.include_pulse {
            let pulse_track = self.generate_pulse_track(&mut midi_file)?;
            midi_file.add_completed_track(pulse_track);
        }

        // Generate performer tracks
        for performer_id in 0..self.performers.len() {
            let performer_track = self.generate_performer_track(performer_id, &mut midi_file)?;
            midi_file.add_completed_track(performer_track);
        }

        Ok(midi_file)
    }

    /// Generate the eighth-note pulse track
    fn generate_pulse_track(&self, midi_file: &mut MidiFile) -> Result<midly::Track<'static>, LobachevskyError> {
        let mut track = midi_file.add_track();

        let total_beats = self.config.duration_minutes * self.config.tempo as f32;
        let high_c = Note::new(crate::PitchClass::C, 6);

        let mut time = 0.0;
        while time < total_beats {
            track.add_note(high_c, time as f64, 0.1, 40, 15);
            time += 0.5; // Eighth note (in quarter-note beats)
        }

        Ok(track.build())
    }

    /// Generate a track for a specific performer
    fn generate_performer_track(
        &mut self,
        performer_id: usize,
        midi_file: &mut MidiFile,
    ) -> Result<midly::Track<'static>, LobachevskyError> {
        let mut track = midi_file.add_track();
        let total_beats = self.config.duration_minutes * self.config.tempo as f32;
        let mut current_time = 0.0;
        let channel = self.performers[performer_id].midi_channel;

        // Simulate the performance
        while current_time < total_beats {
            // Check if listening
            self.performers[performer_id].maybe_start_listening();
            if self.performers[performer_id].is_listening {
                current_time += 0.25; // Wait a quarter beat
                continue;
            }

            // Get current pattern
            let current_pattern = self.performers[performer_id].current_pattern;
            let pattern = self
                .patterns
                .get(current_pattern)
                .ok_or_else(|| LobachevskyError::ParseError {
                    message: format!("Pattern {} not found", current_pattern),
                })?;

            // Apply timing variation
            let timing_ratio = self.performers[performer_id].timing_variation.ratio();
            let effective_pattern = if timing_ratio != 1.0 {
                pattern.with_tempo_ratio(timing_ratio)
            } else {
                pattern.clone()
            };

            // Play current note
            let pattern_position = self.performers[performer_id].pattern_position;
            if pattern_position < effective_pattern.notes.len() {
                let note = &effective_pattern.notes[pattern_position];

                if let Some(pitch) = note.pitch {
                    let octave_transpose = self.performers[performer_id].octave_transpose;
                    let transposed = pitch.transpose(octave_transpose * 12);
                    let base_velocity = self.performers[performer_id].base_velocity;

                    // Main voice
                    track.add_note(
                        transposed,
                        current_time as f64,
                        (note.duration * 4.0) as f64, // Convert to beats
                        base_velocity,
                        channel,
                    );

                    // Canon voices
                    let canon_voices = self.performers[performer_id].canon_voices.clone();
                    for voice in &canon_voices {
                        let canon_time = current_time + voice.offset_beats;
                        if canon_time < total_beats {
                            let canon_velocity = (base_velocity as f32 * voice.velocity_ratio) as u8;

                            track.add_note(
                                transposed,
                                canon_time as f64,
                                (note.duration * 4.0) as f64,
                                canon_velocity,
                                channel,
                            );
                        }
                    }
                }

                // Advance time
                current_time += note.duration * 4.0; // Convert to beats
                self.performers[performer_id].pattern_position += 1;
            } else {
                // Pattern complete, reset position
                self.performers[performer_id].pattern_position = 0;
                
                // Check if we should repeat this pattern
                if self.performers[performer_id].repetitions_remaining > 0 {
                    self.performers[performer_id].repetitions_remaining -= 1;
                    // Continue to play the pattern again
                    continue;
                }
                
                // Pattern fully repeated, maybe advance to next
                let ensemble_median = self.get_ensemble_median();
                self.performers[performer_id].maybe_advance_pattern(ensemble_median, &self.patterns);

                // Check for ending (all at pattern 53)
                if self.all_at_pattern_53() {
                    break;
                }
            }
        }

        Ok(track.build())
    }

    /// Get the median pattern number in the ensemble
    fn get_ensemble_median(&self) -> usize {
        let mut patterns: Vec<usize> = self
            .performers
            .iter()
            .filter(|p| !p.is_listening)
            .map(|p| p.current_pattern)
            .collect();

        patterns.sort();

        if patterns.is_empty() {
            1
        } else {
            patterns[patterns.len() / 2]
        }
    }

    /// Check if all performers have reached pattern 53
    fn all_at_pattern_53(&self) -> bool {
        self.performers.iter().all(|p| p.current_pattern == 53)
    }
}

/// Create an In C performance with default settings
pub fn generate_in_c(patterns: InCPatterns) -> Result<MidiFile, LobachevskyError> {
    let config = InCConfig::default();
    let mut generator = InCGenerator::new(config, patterns);
    generator.generate()
}

/// Create an In C performance with custom configuration
pub fn generate_in_c_with_config(patterns: InCPatterns, config: InCConfig) -> Result<MidiFile, LobachevskyError> {
    let mut generator = InCGenerator::new(config, patterns);
    generator.generate()
}
