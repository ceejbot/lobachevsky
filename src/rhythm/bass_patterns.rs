//! Bass pattern generation for rhythm tracks
//!
//! This module provides bass patterns that work alongside drum patterns,
//! creating rhythmic bass lines that complement the percussion.

use super::Beat;
use crate::{Chord, ChordQuality, Note};

/// Bass voice types for different bass articulations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BassVoice {
    /// Standard bass sound
    Bass,
    /// Short, punchy bass notes
    BassStaccato,
    /// Smooth, connected bass notes
    BassLegato,
    /// Deep sub-bass frequencies
    BassSub,
    /// Sliding/gliding bass notes
    BassSlide,
}

impl BassVoice {
    /// Get the MIDI channel for this bass voice
    /// Bass typically uses channel 2 (index 1)
    pub fn midi_channel(&self) -> u8 {
        match self {
            BassVoice::Bass => 1,
            BassVoice::BassStaccato => 1,
            BassVoice::BassLegato => 2,
            BassVoice::BassSub => 3,
            BassVoice::BassSlide => 4,
        }
    }

    /// Get the default velocity for this bass voice
    pub fn default_velocity(&self) -> u8 {
        match self {
            BassVoice::Bass => 80,
            BassVoice::BassStaccato => 90,
            BassVoice::BassLegato => 70,
            BassVoice::BassSub => 60,
            BassVoice::BassSlide => 75,
        }
    }

    /// Get the default note duration multiplier
    pub fn duration_multiplier(&self) -> f64 {
        match self {
            BassVoice::Bass => 1.0,
            BassVoice::BassStaccato => 0.5, // Shorter notes
            BassVoice::BassLegato => 1.2,   // Slightly overlapping
            BassVoice::BassSub => 1.5,      // Longer sustained notes
            BassVoice::BassSlide => 0.9,    // Leave room for slide
        }
    }
}

/// Bass event combining timing, voice, and pitch information
#[derive(Debug, Clone)]
pub struct BassEvent {
    pub beat: Beat,
    pub voice: BassVoice,
    pub note: Note,
    pub velocity: u8,
    pub duration: f64,
}

/// Trait for bass patterns that generate rhythmic bass lines
pub trait BassPattern: Send + Sync {
    /// Generate bass events for a given number of bars with chord progression
    fn generate(&self, bars: usize, chords: &[Chord]) -> Vec<BassEvent>;

    /// Get a descriptive name for this pattern
    fn name(&self) -> String;
}

/// Euclidean bass pattern - creates evenly distributed bass hits
pub struct EuclideanBassPattern {
    voice: BassVoice,
    hits: usize,
    steps: usize,
    rotation: usize,
    velocity: u8,
    note_pattern: Vec<String>, // e.g., ["root", "fifth", "octave"]
}

impl EuclideanBassPattern {
    pub fn new(voice: BassVoice, hits: usize, steps: usize) -> Self {
        Self {
            voice,
            hits,
            steps,
            rotation: 0,
            velocity: voice.default_velocity(),
            note_pattern: vec!["root".to_string()],
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

    pub fn with_note_pattern(mut self, pattern: Vec<String>) -> Self {
        self.note_pattern = pattern;
        self
    }

    fn get_note_from_pattern(&self, pattern_str: &str, chord: &Chord, octave: i8) -> Note {
        let is_major = matches!(
            chord.quality,
            ChordQuality::Major | ChordQuality::MajorSeventh | ChordQuality::DominantSeventh
        );
        match pattern_str {
            "root" => Note::new(chord.root, octave),
            "third" => Note::new(chord.root.transpose(if is_major { 4 } else { 3 }), octave),
            "fifth" => Note::new(chord.root.transpose(7), octave),
            "octave" => Note::new(chord.root, octave + 1),
            "seventh" => Note::new(chord.root.transpose(if is_major { 11 } else { 10 }), octave),
            _ => Note::new(chord.root, octave), // Default to root
        }
    }
}

impl BassPattern for EuclideanBassPattern {
    fn generate(&self, bars: usize, chords: &[Chord]) -> Vec<BassEvent> {
        use crate::euclidean::{Breshenham, EuclideanRhythm};

        let mut events = Vec::new();

        if self.steps == 0 {
            return events;
        }

        let hits = self.hits.min(self.steps);
        let mut rotated = Breshenham::generate(self.steps, hits).unwrap_or_else(|_| vec![false; self.steps]);
        if self.rotation > 0 && !rotated.is_empty() {
            let rot = self.rotation % rotated.len();
            rotated.rotate_left(rot);
        }

        let beats_per_bar = 4.0;
        let total_beats = bars as f64 * beats_per_bar;
        let chord_duration = total_beats / chords.len() as f64;

        for bar in 0..bars {
            for (step, rotated_event) in rotated.iter().enumerate().take(self.steps) {
                // for step in 0..self.steps {
                if *rotated_event {
                    let beat_pos = bar as f64 * beats_per_bar + (step as f64 / self.steps as f64) * beats_per_bar;
                    let chord_index = (beat_pos / chord_duration) as usize;

                    if chord_index < chords.len() {
                        let chord = &chords[chord_index];
                        let pattern_index = step % self.note_pattern.len();
                        let note = self.get_note_from_pattern(&self.note_pattern[pattern_index], chord, 2);

                        events.push(BassEvent {
                            beat: Beat(beat_pos),
                            voice: self.voice,
                            note,
                            velocity: self.velocity,
                            duration: 1.0 / self.steps as f64 * beats_per_bar * self.voice.duration_multiplier(),
                        });
                    }
                }
            }
        }

        events
    }

    fn name(&self) -> String {
        format!("Euclidean Bass {}/{}", self.hits, self.steps)
    }
}

/// Probability-based bass pattern
pub struct ProbabilityBassPattern {
    voice: BassVoice,
    points: Vec<(Beat, f64, String)>, // (beat, probability, note_type)
    pub velocity_range: (u8, u8),
}

impl ProbabilityBassPattern {
    pub fn new(voice: BassVoice) -> Self {
        Self {
            voice,
            points: Vec::new(),
            velocity_range: (voice.default_velocity() - 10, voice.default_velocity() + 10),
        }
    }

    pub fn add_point(mut self, beat: Beat, probability: f64, note_type: String) -> Self {
        self.points.push((beat, probability, note_type));
        self
    }

    fn get_note_from_type(&self, note_type: &str, chord: &Chord, octave: i8) -> Note {
        let is_major = matches!(
            chord.quality,
            ChordQuality::Major | ChordQuality::MajorSeventh | ChordQuality::DominantSeventh
        );
        match note_type {
            "root" => Note::new(chord.root, octave),
            "third" => Note::new(chord.root.transpose(if is_major { 4 } else { 3 }), octave),
            "fifth" => Note::new(chord.root.transpose(7), octave),
            "octave" => Note::new(chord.root, octave + 1),
            "seventh" => Note::new(chord.root.transpose(if is_major { 11 } else { 10 }), octave),
            _ => Note::new(chord.root, octave),
        }
    }
}

impl BassPattern for ProbabilityBassPattern {
    fn generate(&self, bars: usize, chords: &[Chord]) -> Vec<BassEvent> {
        let mut events = Vec::new();

        let beats_per_bar = 4.0;
        let total_beats = bars as f64 * beats_per_bar;
        let chord_duration = total_beats / chords.len() as f64;

        for bar in 0..bars {
            for (beat_offset, probability, note_type) in &self.points {
                if fastrand::f64() < *probability {
                    let beat_pos = bar as f64 * beats_per_bar + beat_offset.0;
                    let chord_index = (beat_pos / chord_duration) as usize;

                    if chord_index < chords.len() {
                        let chord = &chords[chord_index];
                        let note = self.get_note_from_type(note_type, chord, 2);
                        let velocity = fastrand::u8(self.velocity_range.0..=self.velocity_range.1);

                        events.push(BassEvent {
                            beat: Beat(beat_pos),
                            voice: self.voice,
                            note,
                            velocity,
                            duration: 0.25 * self.voice.duration_multiplier(),
                        });
                    }
                }
            }
        }

        events.sort_by(|a, b| a.beat.0.partial_cmp(&b.beat.0).unwrap_or(std::cmp::Ordering::Equal));
        events
    }

    fn name(&self) -> String {
        format!("Probability Bass ({} points)", self.points.len())
    }
}

/// Sequence bass pattern - plays a specific sequence of notes
pub struct SequenceBassPattern {
    voice: BassVoice,
    sequence: Vec<String>, // e.g., ["root", "root", "fifth", "octave"]
    note_duration: f64,
    velocity_pattern: Vec<u8>,
}

impl SequenceBassPattern {
    pub fn new(voice: BassVoice, sequence: Vec<String>) -> Self {
        Self {
            voice,
            sequence,
            note_duration: 0.25, // Default to 16th notes
            velocity_pattern: vec![voice.default_velocity()],
        }
    }

    pub fn with_note_duration(mut self, duration: f64) -> Self {
        self.note_duration = duration;
        self
    }

    pub fn with_velocity_pattern(mut self, pattern: Vec<u8>) -> Self {
        self.velocity_pattern = pattern;
        self
    }

    fn get_note_from_sequence(&self, seq_item: &str, chord: &Chord, octave: i8) -> Note {
        let is_major = matches!(
            chord.quality,
            ChordQuality::Major | ChordQuality::MajorSeventh | ChordQuality::DominantSeventh
        );
        match seq_item {
            "root" => Note::new(chord.root, octave),
            "third" => Note::new(chord.root.transpose(if is_major { 4 } else { 3 }), octave),
            "fifth" => Note::new(chord.root.transpose(7), octave),
            "octave" => Note::new(chord.root, octave + 1),
            "seventh" => Note::new(chord.root.transpose(if is_major { 11 } else { 10 }), octave),
            _ => Note::new(chord.root, octave),
        }
    }
}

impl BassPattern for SequenceBassPattern {
    fn generate(&self, bars: usize, chords: &[Chord]) -> Vec<BassEvent> {
        let mut events = Vec::new();
        let beats_per_bar = 4.0;
        let total_beats = bars as f64 * beats_per_bar;
        let chord_duration = total_beats / chords.len() as f64;

        let notes_per_bar = (beats_per_bar / self.note_duration) as usize;
        let total_notes = notes_per_bar * bars;

        for note_index in 0..total_notes {
            let beat_pos = note_index as f64 * self.note_duration;
            let chord_index = (beat_pos / chord_duration) as usize;

            if chord_index < chords.len() {
                let chord = &chords[chord_index];
                let seq_index = note_index % self.sequence.len();
                let vel_index = note_index % self.velocity_pattern.len();

                let note = self.get_note_from_sequence(&self.sequence[seq_index], chord, 2);
                let velocity = self.velocity_pattern[vel_index];

                events.push(BassEvent {
                    beat: Beat(beat_pos),
                    voice: self.voice,
                    note,
                    velocity,
                    duration: self.note_duration * self.voice.duration_multiplier(),
                });
            }
        }

        events
    }

    fn name(&self) -> String {
        format!("Sequence Bass ({} steps)", self.sequence.len())
    }
}
