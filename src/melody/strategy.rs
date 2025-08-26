//! Melody generation

use std::fmt::Display;

use rand;
use rand::seq::IndexedRandom;

use crate::melody::MelodyMarkov;
use crate::{Chord, Note};

/// Melody generation strategies
#[derive(Debug, Clone)]
pub enum MelodyStrategy {
    /// Follow chord tones
    ChordTones,
    /// Arpeggiate the chord
    Arpeggio,
    /// Use common tones between chords
    CommonTones,
    /// Stepwise motion within a scale
    Stepwise,
    /// Mixed approach
    Mixed,
    /// Markov chain-based generation (order, use_intervals)
    Markov(usize, bool),
}

impl Display for MelodyStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MelodyStrategy::ChordTones => write!(f, "chord_tones"),
            MelodyStrategy::Arpeggio => write!(f, "arpeggio"),
            MelodyStrategy::CommonTones => write!(f, "stepwise"),
            MelodyStrategy::Stepwise => write!(f, "common_tones"),
            MelodyStrategy::Mixed => write!(f, "mixed"),
            MelodyStrategy::Markov(order, intervals) => {
                write!(f, "markov(order={}, intervals={})", order, intervals)
            }
        }
    }
}

impl From<&str> for MelodyStrategy {
    fn from(value: &str) -> Self {
        let value = value.to_lowercase();

        // Check for Markov patterns like "markov", "markov_2", "markov_intervals"
        if value.starts_with("markov") {
            let parts: Vec<&str> = value.split('_').collect();

            let order = if parts.len() > 1 {
                parts[1].parse().unwrap_or(2)
            } else {
                2
            };

            let use_intervals = parts.contains(&"intervals") || parts.contains(&"interval");

            return MelodyStrategy::Markov(order, use_intervals);
        }

        match value.as_str() {
            "chord_tones" => MelodyStrategy::ChordTones,
            "arpeggio" => MelodyStrategy::Arpeggio,
            "stepwise" => MelodyStrategy::Stepwise,
            "common_tones" => MelodyStrategy::CommonTones,
            "mixed" => MelodyStrategy::Mixed,
            _ => MelodyStrategy::Mixed,
        }
    }
}

/// Melody generator
pub struct MelodyGenerator {
    strategy: MelodyStrategy,
    octave: i8,
    note_duration: f64,
}

impl MelodyGenerator {
    pub fn new(strategy: MelodyStrategy) -> Self {
        MelodyGenerator {
            strategy,
            octave: 5,
            note_duration: 0.5,
        }
    }

    pub fn with_octave(mut self, octave: i8) -> Self {
        self.octave = octave;
        self
    }

    pub fn with_note_duration(mut self, duration: f64) -> Self {
        self.note_duration = duration;
        self
    }

    /// Generate a melody over a chord progression
    pub fn generate(&self, progression: &[Chord], notes_per_chord: usize) -> Vec<(Note, f64, f64)> {
        // Handle Markov generation differently - it needs the full progression
        if let MelodyStrategy::Markov(order, use_intervals) = &self.strategy {
            return self.generate_markov_melody(progression, notes_per_chord, *order, *use_intervals);
        }

        let mut melody = Vec::new();
        let mut beat_position = 0.0;

        for (chord_idx, chord) in progression.iter().enumerate() {
            let chord_notes = self.generate_for_chord(*chord, notes_per_chord, chord_idx);

            for note in chord_notes {
                melody.push((note, beat_position, self.note_duration));
                beat_position += self.note_duration;
            }
        }

        melody
    }

    fn generate_markov_melody(
        &self,
        progression: &[Chord],
        notes_per_chord: usize,
        order: usize,
        use_intervals: bool,
    ) -> Vec<(Note, f64, f64)> {
        let mut melody_markov = MelodyMarkov::new(order, use_intervals);

        // Train on the progression itself
        melody_markov.train_from_chords(progression);

        // Generate melody
        let mut rng = rand::rng();
        let notes = melody_markov.generate_melody(progression, notes_per_chord, self.octave, &mut rng);

        // Convert to timed notes
        let mut timed_melody = Vec::new();
        let mut beat_position = 0.0;

        for note in notes {
            timed_melody.push((note, beat_position, self.note_duration));
            beat_position += self.note_duration;
        }

        timed_melody
    }

    fn generate_for_chord(&self, chord: Chord, num_notes: usize, chord_idx: usize) -> Vec<Note> {
        match &self.strategy {
            MelodyStrategy::ChordTones => self.chord_tones_melody(chord, num_notes),
            MelodyStrategy::Arpeggio => self.arpeggio_melody(chord, num_notes, chord_idx),
            MelodyStrategy::CommonTones => self.chord_tones_melody(chord, num_notes), // Simplified
            MelodyStrategy::Stepwise => self.stepwise_melody(chord, num_notes),
            MelodyStrategy::Mixed => self.mixed_melody(chord, num_notes, chord_idx),
            MelodyStrategy::Markov(_order, _use_intervals) => {
                // For individual chord generation, fall back to chord tones
                // Full Markov generation happens at the progression level
                self.chord_tones_melody(chord, num_notes)
            }
        }
    }

    fn chord_tones_melody(&self, chord: Chord, num_notes: usize) -> Vec<Note> {
        let mut rng = rand::rng();

        let chord_notes = chord.notes(self.octave);
        let mut melody = Vec::new();

        for _ in 0..num_notes {
            if let Some(note) = chord_notes.choose(&mut rng) {
                melody.push(*note);
            }
        }

        melody
    }

    fn arpeggio_melody(&self, chord: Chord, num_notes: usize, chord_idx: usize) -> Vec<Note> {
        let chord_notes = chord.notes(self.octave);
        let mut melody = Vec::new();

        // Different arpeggio patterns based on chord index
        let pattern = match chord_idx % 4 {
            0 => vec![0, 1, 2, 1], // Up and back
            1 => vec![2, 1, 0, 1], // Down and back
            2 => vec![0, 2, 1, 2], // Skip pattern
            _ => vec![0, 1, 2, 2], // Emphasis on top
        };

        for i in 0..num_notes {
            let note_idx = pattern[i % pattern.len()] % chord_notes.len();
            melody.push(chord_notes[note_idx]);
        }

        melody
    }

    fn stepwise_melody(&self, chord: Chord, num_notes: usize) -> Vec<Note> {
        let mut melody = Vec::new();
        let root = Note::new(chord.root, self.octave);

        // Create a simple stepwise line
        let mut current = root;
        for i in 0..num_notes {
            melody.push(current);

            // Move up or down by step
            let direction = if i % 4 < 2 { 1 } else { -1 };
            current = current.transpose(direction);
        }

        melody
    }

    fn mixed_melody(&self, chord: Chord, num_notes: usize, chord_idx: usize) -> Vec<Note> {
        // Mix different strategies based on position
        if chord_idx % 3 == 0 {
            self.arpeggio_melody(chord, num_notes, chord_idx)
        } else if chord_idx % 3 == 1 {
            self.chord_tones_melody(chord, num_notes)
        } else {
            self.stepwise_melody(chord, num_notes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_generate_melodies() {
        let progression = vec![Chord::c_major(), Chord::a_minor()];
        let generator = MelodyGenerator::new(MelodyStrategy::Arpeggio);

        let melody = generator.generate(&progression, 4);
        assert_eq!(melody.len(), 8); // 4 notes per chord * 2 chords
    }
}
