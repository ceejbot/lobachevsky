//! Progression and melody generation tools

use rand::seq::IndexedRandom;

use crate::core::{Chord, Mode, Note, PitchClass};
use crate::theory::{ModalNeoRiemannian, NeoRiemannian, Transform};

/// Transformation engine type
pub enum TransformEngine {
    Free(NeoRiemannian),
    Modal(ModalNeoRiemannian),
}

impl TransformEngine {
    fn transform(&self, chord: Chord, transform: &Transform) -> Option<Chord> {
        match self {
            TransformEngine::Free(engine) => engine.transform(chord, transform),
            TransformEngine::Modal(engine) => engine.transform(chord, transform),
        }
    }

    #[allow(dead_code)]
    fn apply_sequence(&self, start: Chord, transforms: &[Transform]) -> Vec<Chord> {
        match self {
            TransformEngine::Free(engine) => engine.apply_sequence(start, transforms),
            TransformEngine::Modal(engine) => engine.apply_sequence(start, transforms),
        }
    }
}

/// Builder for creating chord progressions
pub struct ProgressionBuilder {
    start_chord: Option<Chord>,
    transforms: Vec<Transform>,
    length: usize,
    return_to_start: bool,
    transformer: TransformEngine,
    modal_constraints: Option<(Mode, PitchClass)>,
}

impl ProgressionBuilder {
    /// Create a new progression builder
    pub fn new() -> Self {
        ProgressionBuilder {
            start_chord: None,
            transforms: Vec::new(),
            length: 8,
            return_to_start: false,
            transformer: TransformEngine::Free(NeoRiemannian::new()),
            modal_constraints: None,
        }
    }

    /// Set the starting chord
    pub fn start(mut self, chord: Chord) -> Self {
        self.start_chord = Some(chord);
        self
    }

    /// Add a single transformation to the pattern
    pub fn add_transform(mut self, transform: Transform) -> Self {
        self.transforms.push(transform);
        self
    }

    /// Set a repeating pattern of transformations
    pub fn pattern(mut self, transforms: &[Transform]) -> Self {
        self.transforms = transforms.to_vec();
        self
    }

    /// Set the length of the progression (number of chords)
    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Make the progression return to the starting chord
    pub fn with_return(mut self) -> Self {
        self.return_to_start = true;
        self
    }

    /// Constrain transformations to a specific mode
    pub fn with_mode(mut self, mode: Mode, tonic: PitchClass) -> Self {
        self.modal_constraints = Some((mode, tonic));
        self.transformer = TransformEngine::Modal(ModalNeoRiemannian::new(mode, tonic));
        self
    }

    /// Get information about modal constraints if any
    pub fn modal_info(&self) -> Option<(Mode, PitchClass)> {
        self.modal_constraints
    }

    /// Build the progression
    pub fn build(&self) -> Vec<Chord> {
        let start = self.start_chord.unwrap_or(Chord::c_major());

        if self.transforms.is_empty() {
            return vec![start; self.length];
        }

        let mut progression = vec![start];
        let mut current = start;

        let target_length = if self.return_to_start {
            self.length - 1 // Leave room for return
        } else {
            self.length
        };

        for i in 0..target_length.saturating_sub(1) {
            let transform = &self.transforms[i % self.transforms.len()];
            if let Some(next) = self.transformer.transform(current, transform) {
                progression.push(next);
                current = next;
            } else {
                // If transformation fails, repeat current chord
                progression.push(current);
            }
        }

        if self.return_to_start && progression.len() < self.length {
            progression.push(start);
        }

        progression
    }

    /// Generate a random walk through transformations
    pub fn random_walk(mut self, length: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::rng();

        let transforms = [Transform::P, Transform::R, Transform::L];
        let mut pattern = Vec::new();

        for _ in 0..length {
            pattern.push(transforms[rng.random_range(0..transforms.len())].clone());
        }

        self.transforms = pattern;
        self.length = length;
        self
    }
}

impl Default for ProgressionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Melody generation strategies
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

    fn generate_for_chord(&self, chord: Chord, num_notes: usize, chord_idx: usize) -> Vec<Note> {
        match self.strategy {
            MelodyStrategy::ChordTones => self.chord_tones_melody(chord, num_notes),
            MelodyStrategy::Arpeggio => self.arpeggio_melody(chord, num_notes, chord_idx),
            MelodyStrategy::CommonTones => self.chord_tones_melody(chord, num_notes), // Simplified
            MelodyStrategy::Stepwise => self.stepwise_melody(chord, num_notes),
            MelodyStrategy::Mixed => self.mixed_melody(chord, num_notes, chord_idx),
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

/// Hexatonic cycle explorer
pub struct HexatonicExplorer {
    transformer: NeoRiemannian,
}

impl Default for HexatonicExplorer {
    fn default() -> Self {
        Self::new()
    }
}

impl HexatonicExplorer {
    pub fn new() -> Self {
        HexatonicExplorer {
            transformer: NeoRiemannian::new(),
        }
    }

    /// Get a specific hexatonic cycle
    pub fn get_cycle(&self, cycle_type: HexatonicCycle, starting_chord: Chord) -> Option<Vec<Chord>> {
        let pattern = match cycle_type {
            HexatonicCycle::Northern => vec![Transform::P, Transform::L],
            HexatonicCycle::Western => vec![Transform::P, Transform::R],
            HexatonicCycle::Eastern => vec![Transform::L, Transform::R],
            HexatonicCycle::Augmented => vec![Transform::P, Transform::L, Transform::P],
        };

        self.transformer.hexatonic_cycle(starting_chord, &pattern)
    }

    /// Find all cycles of a given type
    pub fn find_all_cycles(&self, cycle_type: HexatonicCycle) -> Vec<Vec<Chord>> {
        let all_cycles = self.transformer.all_hexatonic_cycles();

        let key = match cycle_type {
            HexatonicCycle::Northern => "Northern (PL)",
            HexatonicCycle::Western => "Western (PR)",
            HexatonicCycle::Eastern => "Eastern (LR)",
            HexatonicCycle::Augmented => return Vec::new(), // Special case
        };

        all_cycles.get(key).cloned().unwrap_or_default()
    }
}

/// Types of hexatonic cycles
#[derive(Debug, Clone, Copy)]
pub enum HexatonicCycle {
    Northern,  // PL cycle
    Western,   // PR cycle
    Eastern,   // LR cycle
    Augmented, // PLP cycle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progression_builder() {
        let progression = ProgressionBuilder::new()
            .start(Chord::f_major())
            .pattern(&[Transform::P, Transform::R, Transform::L])
            .length(4)
            .build();

        assert_eq!(progression.len(), 4);
        assert_eq!(progression[0], Chord::f_major());
    }

    #[test]
    fn progression_with_return() {
        let progression = ProgressionBuilder::new()
            .start(Chord::c_major())
            .pattern(&[Transform::R, Transform::L])
            .length(4)
            .with_return()
            .build();

        assert_eq!(progression.len(), 4);
        assert_eq!(progression[0], Chord::c_major());
        assert_eq!(progression[3], Chord::c_major());
    }

    #[test]
    fn can_generate_melodies() {
        let progression = vec![Chord::c_major(), Chord::a_minor()];
        let generator = MelodyGenerator::new(MelodyStrategy::Arpeggio);

        let melody = generator.generate(&progression, 4);
        assert_eq!(melody.len(), 8); // 4 notes per chord * 2 chords
    }
}
