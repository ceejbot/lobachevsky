//! Melody generation

use std::fmt::Display;

use crate::melody::MelodyMarkov;
use crate::{Chord, Note};

/// Electronic melody generation strategies
#[derive(Debug, Clone)]
pub enum MelodyStrategy {
    /// Soaring lead synthesizer lines with filter sweeps
    LeadSynth,
    /// Fast arpeggiated sequences, classic electronic style
    Arpeggiated,
    /// Punchy rhythmic chord stabs and accents
    RhythmicStabs,
    /// Sustained textural pads and atmospheric sounds
    TexturalPads,
    /// Percussive pluck sequences with staccato articulation
    PluckSequence,
    /// Deep bass-register lead lines
    BassLead,
    /// Markov chain-based generation (order, use_intervals)
    Markov(usize, bool),
}

impl Display for MelodyStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MelodyStrategy::LeadSynth => write!(f, "lead_synth"),
            MelodyStrategy::Arpeggiated => write!(f, "arpeggiated"),
            MelodyStrategy::RhythmicStabs => write!(f, "rhythmic_stabs"),
            MelodyStrategy::TexturalPads => write!(f, "textural_pads"),
            MelodyStrategy::PluckSequence => write!(f, "pluck_sequence"),
            MelodyStrategy::BassLead => write!(f, "bass_lead"),
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
            "lead_synth" => MelodyStrategy::LeadSynth,
            "arpeggiated" => MelodyStrategy::Arpeggiated,
            "rhythmic_stabs" => MelodyStrategy::RhythmicStabs,
            "textural_pads" => MelodyStrategy::TexturalPads,
            "pluck_sequence" => MelodyStrategy::PluckSequence,
            "bass_lead" => MelodyStrategy::BassLead,
            _ => MelodyStrategy::LeadSynth,
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
        let notes = melody_markov.generate_melody(progression, notes_per_chord, self.octave);

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
            MelodyStrategy::LeadSynth => self.lead_synth_melody(chord, num_notes, chord_idx),
            MelodyStrategy::Arpeggiated => self.arpeggiated_melody(chord, num_notes, chord_idx),
            MelodyStrategy::RhythmicStabs => self.rhythmic_stabs_melody(chord, num_notes, chord_idx),
            MelodyStrategy::TexturalPads => self.textural_pads_melody(chord, num_notes),
            MelodyStrategy::PluckSequence => self.pluck_sequence_melody(chord, num_notes, chord_idx),
            MelodyStrategy::BassLead => self.bass_lead_melody(chord, num_notes, chord_idx),
            MelodyStrategy::Markov(_order, _use_intervals) => {
                // For individual chord generation, fall back to lead synth
                // Full Markov generation happens at the progression level
                self.lead_synth_melody(chord, num_notes, chord_idx)
            }
        }
    }

    /// Lead synth with soaring melodic lines and filter sweeps
    fn lead_synth_melody(&self, chord: Chord, num_notes: usize, _chord_idx: usize) -> Vec<Note> {
        let chord_notes = chord.notes(self.octave);
        let mut melody = Vec::new();

        // Start with chord tone, then create melodic phrases
        let root_note = chord_notes[0];
        melody.push(root_note);

        let mut current = root_note;
        for i in 1..num_notes {
            // Create melodic motion with larger intervals for lead synth character
            let next_note = if i % 4 == 0 {
                // Return to chord tones for stability
                fastrand::choice(&chord_notes).copied().unwrap_or(current)
            } else {
                // Melodic motion with jumps and steps
                let interval = match fastrand::u8(0..=4) {
                    0 => -7, // Down fifth
                    1 => -3, // Down minor third
                    2 => 2,  // Up major second
                    3 => 4,  // Up major third
                    _ => 7,  // Up fifth
                };

                let candidate = current.transpose(interval);
                // Keep in reasonable range for lead
                if candidate.octave >= 4 && candidate.octave <= 7 {
                    candidate
                } else {
                    fastrand::choice(&chord_notes).copied().unwrap_or(current)
                }
            };

            melody.push(next_note);
            current = next_note;
        }

        melody
    }

    /// Fast arpeggiated sequences in classic electronic style
    fn arpeggiated_melody(&self, chord: Chord, num_notes: usize, chord_idx: usize) -> Vec<Note> {
        let chord_notes = chord.notes(self.octave);
        let mut melody = Vec::new();

        // Extended arpeggio patterns across octaves
        let base_pattern = match chord_idx % 6 {
            0 => vec![0, 1, 2, 1], // Classic up-down
            1 => vec![0, 2, 1, 2], // Skip pattern
            2 => vec![2, 1, 0, 1], // Inverted
            3 => vec![0, 1, 2, 0], // Octave return
            4 => vec![1, 0, 2, 1], // Second inversion start
            _ => vec![0, 2, 0, 1], // Root emphasis
        };

        for i in 0..num_notes {
            let base_idx = base_pattern[i % base_pattern.len()];
            let mut note = chord_notes[base_idx % chord_notes.len()];

            // Add octave variations for electronic arpeggio character
            if i % 8 >= 4 {
                note = Note::new(note.pitch_class, note.octave + 1);
            }

            melody.push(note);
        }

        melody
    }

    /// Punchy rhythmic chord stabs and accents
    fn rhythmic_stabs_melody(&self, chord: Chord, num_notes: usize, chord_idx: usize) -> Vec<Note> {
        let chord_notes = chord.notes(self.octave);
        let mut melody = Vec::new();

        // Rhythmic stab patterns - not every beat has a note
        let stab_pattern = match chord_idx % 4 {
            0 => vec![true, false, true, false], // On beats 1 and 3
            1 => vec![true, false, false, true], // Syncopated
            2 => vec![false, true, true, false], // Off-beat emphasis
            _ => vec![true, true, false, true],  // Dense pattern
        };

        for i in 0..num_notes {
            if stab_pattern[i % stab_pattern.len()] {
                // Use full chord or chord subset for stabs
                let stab_note = if fastrand::f64() < 0.7 {
                    chord_notes[0] // Root emphasis
                } else {
                    fastrand::choice(&chord_notes).copied().unwrap_or(chord_notes[0])
                };
                melody.push(stab_note);
            } else {
                // Rest or sustained note (use root at low velocity conceptually)
                melody.push(chord_notes[0]);
            }
        }

        melody
    }

    /// Sustained textural pads and atmospheric sounds
    fn textural_pads_melody(&self, chord: Chord, _num_notes: usize) -> Vec<Note> {
        let chord_notes = chord.notes(self.octave);
        let mut melody = Vec::new();

        // Pads sustain the full chord - create a simple voicing
        // Use root position or inversion based on voice leading
        let pad_notes = vec![
            chord_notes[0], // Root
            chord_notes[1], // Third (typically)
            chord_notes[2], // Fifth (typically)
        ];

        // For pads, we typically want sustained notes
        // Return just the chord tones to be sustained
        melody.extend(pad_notes);

        melody
    }

    /// Percussive pluck sequences with staccato articulation
    fn pluck_sequence_melody(&self, chord: Chord, num_notes: usize, chord_idx: usize) -> Vec<Note> {
        let chord_notes = chord.notes(self.octave);
        let mut melody = Vec::new();

        // Pluck sequences often use single notes in rhythmic patterns
        let sequence_pattern = match chord_idx % 5 {
            0 => vec![0, 0, 1, 0], // Root heavy
            1 => vec![0, 2, 1, 2], // Triad pattern
            2 => vec![1, 0, 1, 0], // Third emphasis
            3 => vec![2, 1, 0, 1], // Descending
            _ => vec![0, 1, 2, 1], // Classic pattern
        };

        for i in 0..num_notes {
            let note_idx = sequence_pattern[i % sequence_pattern.len()];
            let mut pluck_note = chord_notes[note_idx % chord_notes.len()];

            // Add occasional octave jumps for pluck character
            if fastrand::f64() < 0.2 {
                pluck_note = Note::new(pluck_note.pitch_class, pluck_note.octave + 1);
            }

            melody.push(pluck_note);
        }

        melody
    }

    /// Deep bass-register lead lines
    fn bass_lead_melody(&self, chord: Chord, num_notes: usize, chord_idx: usize) -> Vec<Note> {
        let bass_octave = (self.octave - 2).max(1); // Force bass register
        let chord_notes = chord.notes(bass_octave);
        let mut melody = Vec::new();

        // Bass leads focus on root motion and fifths
        let mut current = chord_notes[0]; // Start on root
        melody.push(current);

        for i in 1..num_notes {
            let next_note = match (i + chord_idx) % 6 {
                0 | 3 => chord_notes[0], // Root
                1 | 4 => {
                    // Perfect fifth
                    Note::new(chord.root.transpose(7), bass_octave)
                }
                2 => chord_notes.get(2).copied().unwrap_or(chord_notes[0]), // Fifth from chord
                _ => {
                    // Stepwise motion in bass register
                    let step = if fastrand::bool() { 1 } else { -1 };
                    current.transpose(step)
                }
            };

            melody.push(next_note);
            current = next_note;
        }

        melody
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_generate_melodies() {
        let progression = vec![Chord::c_major(), Chord::a_minor()];
        let generator = MelodyGenerator::new(MelodyStrategy::Arpeggiated);

        let melody = generator.generate(&progression, 4);
        assert_eq!(melody.len(), 8); // 4 notes per chord * 2 chords
    }
}
