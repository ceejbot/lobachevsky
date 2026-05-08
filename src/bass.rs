//! Algorithmic bass line generation
//!
//! This module provides various strategies for generating bass lines that
//! complement chord progressions and rhythm patterns.

use std::fmt::Display;

use crate::{Chord, Note, PitchClass};

/// Bass line generation strategies
#[derive(Debug, Clone)]
pub enum BassStrategy {
    /// Follow the root note of each chord
    Root,
    /// Walking bass line with passing tones
    Walking,
    /// Pedal point - sustain a single note
    Pedal(PitchClass),
    /// Rhythmic bass following the kick pattern
    Rhythmic,
    /// Independent melodic counterpoint
    Counterpoint,
    /// Alternating between root and fifth
    RootFifth,
}

impl Display for BassStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BassStrategy::Root => write!(f, "root"),
            BassStrategy::Walking => write!(f, "walking"),
            BassStrategy::Pedal(note) => write!(f, "pedal({})", note),
            BassStrategy::Rhythmic => write!(f, "rhythmic"),
            BassStrategy::Counterpoint => write!(f, "counterpoint"),
            BassStrategy::RootFifth => write!(f, "root_fifth"),
        }
    }
}

impl From<&str> for BassStrategy {
    fn from(value: &str) -> Self {
        let value = value.to_lowercase();

        // Check for pedal with specified note
        if value.starts_with("pedal") {
            if let Some(note_part) = value.split('_').nth(1)
                && let Ok(pitch_class) = PitchClass::try_from(note_part.to_uppercase().as_str())
            {
                return BassStrategy::Pedal(pitch_class);
            }
            // Default to pedal on C if no note specified
            return BassStrategy::Pedal(PitchClass::C);
        }

        match value.as_str() {
            "root" => BassStrategy::Root,
            "walking" => BassStrategy::Walking,
            "rhythmic" => BassStrategy::Rhythmic,
            "counterpoint" => BassStrategy::Counterpoint,
            "root_fifth" => BassStrategy::RootFifth,
            _ => BassStrategy::Root,
        }
    }
}

/// Bass line generator
#[derive(Debug, Clone)]
pub struct BassGenerator {
    strategy: BassStrategy,
    octave: i8,
    note_duration: f64,
}

impl Default for BassGenerator {
    fn default() -> Self {
        Self::new(BassStrategy::Root)
    }
}

impl BassGenerator {
    /// Create a new bass generator with the specified strategy
    pub fn new(strategy: BassStrategy) -> Self {
        Self {
            strategy,
            octave: 2,          // Bass typically in octave 2
            note_duration: 1.0, // Whole note by default
        }
    }

    /// Set the octave for bass notes
    pub fn with_octave(mut self, octave: i8) -> Self {
        self.octave = octave;
        self
    }

    /// Set the note duration
    pub fn with_note_duration(mut self, duration: f64) -> Self {
        self.note_duration = duration;
        self
    }

    /// Generate a bass line for a chord progression
    pub fn generate(&self, progression: &[Chord], notes_per_chord: usize) -> Vec<(Note, f64, f64)> {
        match &self.strategy {
            BassStrategy::Root => self.generate_root_bass(progression, notes_per_chord),
            BassStrategy::Walking => self.generate_walking_bass(progression, notes_per_chord),
            BassStrategy::Pedal(pitch_class) => self.generate_pedal_bass(*pitch_class, progression, notes_per_chord),
            BassStrategy::Rhythmic => self.generate_rhythmic_bass(progression, notes_per_chord),
            BassStrategy::Counterpoint => self.generate_counterpoint_bass(progression, notes_per_chord),
            BassStrategy::RootFifth => self.generate_root_fifth_bass(progression, notes_per_chord),
        }
    }

    /// Generate root note bass line
    fn generate_root_bass(&self, progression: &[Chord], notes_per_chord: usize) -> Vec<(Note, f64, f64)> {
        let mut bass_line = Vec::new();
        let mut beat_position = 0.0;

        for chord in progression {
            let root_note = Note::new(chord.root, self.octave);

            for _ in 0..notes_per_chord {
                bass_line.push((root_note, beat_position, self.note_duration));
                beat_position += self.note_duration;
            }
        }

        bass_line
    }

    /// Generate walking bass line with passing tones
    fn generate_walking_bass(&self, progression: &[Chord], notes_per_chord: usize) -> Vec<(Note, f64, f64)> {
        let mut bass_line = Vec::new();
        let mut beat_position = 0.0;

        for (chord_idx, chord) in progression.iter().enumerate() {
            let current_root = Note::new(chord.root, self.octave);

            // Always start with the root note
            bass_line.push((current_root, beat_position, self.note_duration));
            beat_position += self.note_duration;

            // Fill remaining notes with walking motion
            for note_idx in 1..notes_per_chord {
                let target_note = if chord_idx + 1 < progression.len() {
                    // Walk towards next chord's root
                    Note::new(progression[chord_idx + 1].root, self.octave)
                } else {
                    // Last chord - walk within current chord
                    current_root
                };

                let walking_note = if note_idx == notes_per_chord - 1 {
                    // Last note - use passing tone or approach note
                    self.get_approach_note(current_root, target_note)
                } else {
                    // Middle notes - use chord tones or chromatic approach
                    if fastrand::f64() < 0.6 {
                        // Use chord tone
                        let chord_tones = chord.notes(self.octave);
                        fastrand::choice(&chord_tones).copied().unwrap_or(current_root)
                    } else {
                        // Use chromatic passing tone
                        self.get_chromatic_passing_tone(current_root, target_note)
                    }
                };

                bass_line.push((walking_note, beat_position, self.note_duration));
                beat_position += self.note_duration;
            }
        }

        bass_line
    }

    /// Generate pedal point bass line
    fn generate_pedal_bass(
        &self,
        pedal_note: PitchClass,
        progression: &[Chord],
        notes_per_chord: usize,
    ) -> Vec<(Note, f64, f64)> {
        let mut bass_line = Vec::new();
        let mut beat_position = 0.0;
        let pedal = Note::new(pedal_note, self.octave);

        for _ in progression {
            for _ in 0..notes_per_chord {
                bass_line.push((pedal, beat_position, self.note_duration));
                beat_position += self.note_duration;
            }
        }

        bass_line
    }

    /// Generate rhythmic bass following typical patterns
    fn generate_rhythmic_bass(&self, progression: &[Chord], notes_per_chord: usize) -> Vec<(Note, f64, f64)> {
        let mut bass_line = Vec::new();
        let mut beat_position = 0.0;

        for chord in progression {
            let root_note = Note::new(chord.root, self.octave);
            let fifth_note = Note::new(chord.root.transpose(7), self.octave); // Perfect fifth

            for note_idx in 0..notes_per_chord {
                let note = match note_idx % 4 {
                    0 => root_note, // Strong beat - root
                    1 => {
                        if fastrand::f64() < 0.3 {
                            root_note // Sometimes repeat root
                        } else {
                            continue; // Often rest
                        }
                    }
                    2 => {
                        if fastrand::f64() < 0.7 {
                            fifth_note // Off-beat emphasis
                        } else {
                            root_note
                        }
                    }
                    3 => {
                        if fastrand::f64() < 0.4 {
                            root_note // Syncopation
                        } else {
                            continue; // Rest
                        }
                    }
                    _ => root_note,
                };

                bass_line.push((note, beat_position, self.note_duration));
                beat_position += self.note_duration;
            }
        }

        bass_line
    }

    /// Generate counterpoint bass line
    fn generate_counterpoint_bass(&self, progression: &[Chord], notes_per_chord: usize) -> Vec<(Note, f64, f64)> {
        let mut bass_line = Vec::new();
        let mut beat_position = 0.0;
        let mut current_note = Note::new(progression[0].root, self.octave);

        for chord in progression {
            let chord_tones = chord.notes(self.octave);

            for _ in 0..notes_per_chord {
                // Choose next note with melodic motion
                let candidates: Vec<Note> = chord_tones
                    .iter()
                    .filter(|&&note| {
                        let interval = (note.to_midi() as i16 - current_note.to_midi() as i16).abs();
                        interval <= 7 // Within a perfect fifth
                    })
                    .cloned()
                    .collect();

                let next_note = if candidates.is_empty() {
                    fastrand::choice(&chord_tones).copied().unwrap_or(current_note)
                } else {
                    fastrand::choice(&candidates).copied().unwrap_or(current_note)
                };

                bass_line.push((next_note, beat_position, self.note_duration));
                beat_position += self.note_duration;
                current_note = next_note;
            }
        }

        bass_line
    }

    /// Generate root-fifth alternating pattern
    fn generate_root_fifth_bass(&self, progression: &[Chord], notes_per_chord: usize) -> Vec<(Note, f64, f64)> {
        let mut bass_line = Vec::new();
        let mut beat_position = 0.0;

        for chord in progression {
            let root_note = Note::new(chord.root, self.octave);
            let fifth_note = Note::new(chord.root.transpose(7), self.octave);

            for note_idx in 0..notes_per_chord {
                let note = if note_idx % 2 == 0 { root_note } else { fifth_note };
                bass_line.push((note, beat_position, self.note_duration));
                beat_position += self.note_duration;
            }
        }

        bass_line
    }

    /// Get an approach note leading to the target
    fn get_approach_note(&self, current: Note, target: Note) -> Note {
        let current_midi = current.to_midi() as i16;
        let target_midi = target.to_midi() as i16;

        // Choose chromatic approach from above or below
        let approach_midi = if fastrand::bool() {
            if target_midi > current_midi {
                target_midi - 1 // Approach from below
            } else {
                target_midi + 1 // Approach from above
            }
        } else {
            // Use scale-wise approach (just choose a nearby note)
            if target_midi > current_midi {
                current_midi + 2 // Step up
            } else {
                current_midi - 2 // Step down
            }
        };

        Note::from_midi(approach_midi.clamp(24, 84) as u8) // Keep in bass range
    }

    /// Get a chromatic passing tone between current and target
    fn get_chromatic_passing_tone(&self, current: Note, target: Note) -> Note {
        let current_midi = current.to_midi() as i16;
        let target_midi = target.to_midi() as i16;

        let passing_midi = if target_midi > current_midi {
            current_midi + fastrand::i16(1..=(target_midi - current_midi).min(3))
        } else if target_midi < current_midi {
            current_midi - fastrand::i16(1..=(current_midi - target_midi).min(3))
        } else {
            // Same note - add slight variation
            current_midi + if fastrand::bool() { 1 } else { -1 }
        };

        Note::from_midi(passing_midi.clamp(24, 84) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chord;

    #[test]
    fn bass_generator_root_strategy() {
        let bass_gen = BassGenerator::new(BassStrategy::Root)
            .with_octave(2)
            .with_note_duration(1.0);

        let chords = vec![Chord::c_major(), Chord::f_major()];
        let bass_line = bass_gen.generate(&chords, 2);

        assert_eq!(bass_line.len(), 4); // 2 chords * 2 notes each

        // Should start with C in octave 2
        assert_eq!(bass_line[0].0.pitch_class, PitchClass::C);
        assert_eq!(bass_line[0].0.octave, 2);

        // Third note should be F (second chord)
        assert_eq!(bass_line[2].0.pitch_class, PitchClass::F);
    }

    #[test]
    fn bass_generator_walking_strategy() {
        let bass_gen = BassGenerator::new(BassStrategy::Walking)
            .with_octave(2)
            .with_note_duration(0.5);

        let chords = vec![Chord::c_major(), Chord::g_major()];
        let bass_line = bass_gen.generate(&chords, 3);

        assert_eq!(bass_line.len(), 6); // 2 chords * 3 notes each

        // Should start with C
        assert_eq!(bass_line[0].0.pitch_class, PitchClass::C);

        // Should have varying notes (not all the same)
        let unique_pitches: std::collections::HashSet<_> =
            bass_line.iter().map(|(note, _, _)| note.pitch_class).collect();
        assert!(unique_pitches.len() > 1);
    }

    #[test]
    fn bass_generator_pedal_strategy() {
        let bass_gen = BassGenerator::new(BassStrategy::Pedal(PitchClass::G)).with_octave(2);

        let chords = vec![Chord::c_major(), Chord::f_major(), Chord::g_major()];
        let bass_line = bass_gen.generate(&chords, 2);

        assert_eq!(bass_line.len(), 6);

        // All notes should be G
        for (note, _, _) in bass_line {
            assert_eq!(note.pitch_class, PitchClass::G);
            assert_eq!(note.octave, 2);
        }
    }

    #[test]
    fn bass_generator_root_fifth_strategy() {
        let bass_gen = BassGenerator::new(BassStrategy::RootFifth).with_octave(2);

        let chords = vec![Chord::c_major()];
        let bass_line = bass_gen.generate(&chords, 4);

        assert_eq!(bass_line.len(), 4);

        // Should alternate between C (root) and G (fifth)
        assert_eq!(bass_line[0].0.pitch_class, PitchClass::C);
        assert_eq!(bass_line[1].0.pitch_class, PitchClass::G);
        assert_eq!(bass_line[2].0.pitch_class, PitchClass::C);
        assert_eq!(bass_line[3].0.pitch_class, PitchClass::G);
    }

    #[test]
    fn bass_strategy_from_string() {
        assert!(matches!(BassStrategy::from("root"), BassStrategy::Root));
        assert!(matches!(BassStrategy::from("walking"), BassStrategy::Walking));
        assert!(matches!(BassStrategy::from("rhythmic"), BassStrategy::Rhythmic));

        if let BassStrategy::Pedal(pc) = BassStrategy::from("pedal_G") {
            assert_eq!(pc, PitchClass::G);
        } else {
            panic!("Expected Pedal(G)");
        }
    }
}
