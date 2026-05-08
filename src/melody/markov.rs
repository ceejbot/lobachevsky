//! Markov chain implementation for melody generation
//!
//! This module provides a flexible Markov chain system that can be trained on
//! musical data to generate new melodies that follow learned patterns.

use std::collections::HashMap;
use std::hash::Hash;

use crate::{Chord, Note};

/// A generic Markov chain that can work with any type T
#[derive(Debug, Clone)]
pub struct MarkovChain<T> {
    /// Order of the Markov chain (how many previous states to consider)
    order: usize,

    /// Transition table: maps state sequences to possible next states with
    /// probabilities
    transitions: HashMap<Vec<T>, Vec<(T, f64)>>,

    /// Starting states for generating new sequences
    starting_states: Vec<Vec<T>>,
}

impl<T> MarkovChain<T>
where
    T: Clone + Eq + Hash,
{
    /// Create a new Markov chain with the specified order
    pub fn new(order: usize) -> Self {
        Self {
            order: order.max(1), // Ensure at least order 1
            transitions: HashMap::new(),
            starting_states: Vec::new(),
        }
    }

    /// Train the Markov chain on a sequence of data
    pub fn train(&mut self, sequence: &[T]) {
        if sequence.len() <= self.order {
            return; // Not enough data to train
        }

        // Record starting state
        if sequence.len() >= self.order {
            self.starting_states.push(sequence[0..self.order].to_vec());
        }

        // Build transition table
        for window in sequence.windows(self.order + 1) {
            let state = window[0..self.order].to_vec();
            let next = window[self.order].clone();

            self.transitions.entry(state).or_default().push((next, 1.0)); // Initial weight of 1.0
        }

        // Normalize probabilities
        self.normalize_probabilities();
    }

    /// Train on multiple sequences
    pub fn train_multiple(&mut self, sequences: &[&[T]]) {
        for sequence in sequences {
            self.train(sequence);
        }
    }

    /// Generate a new sequence of the specified length
    pub fn generate(&self, length: usize) -> Vec<T> {
        if length <= self.order {
            return Vec::new();
        }

        // Start with a random starting state
        let Some(starting) = fastrand::choice(&self.starting_states) else {
            return Vec::new();
        };
        let mut result = starting.clone();

        // Generate remaining notes
        for _ in self.order..length {
            let current_state: Vec<T> = result[result.len() - self.order..].to_vec();

            if let Some(possible_next) = self.transitions.get(&current_state) {
                if let Some(next) = self.weighted_choice(possible_next) {
                    result.push(next);
                } else {
                    // Fallback: pick random starting state
                    if let Some(random_state) = fastrand::choice(&self.starting_states) {
                        result.extend(random_state.iter().cloned());
                    }
                    break;
                }
            } else {
                // No transition found, try to find a partial match
                if !self.find_partial_match_and_continue(&mut result, &current_state) {
                    break;
                }
            }
        }

        result.truncate(length);
        result
    }

    /// Normalize probabilities so they sum to 1.0
    fn normalize_probabilities(&mut self) {
        for transitions in self.transitions.values_mut() {
            // Count occurrences
            let mut counts: HashMap<T, usize> = HashMap::new();
            for (item, _) in transitions.iter() {
                *counts.entry(item.clone()).or_insert(0) += 1;
            }

            // Convert to probabilities
            let total: usize = counts.values().sum();
            transitions.clear();

            for (item, count) in counts {
                let probability = count as f64 / total as f64;
                transitions.push((item, probability));
            }
        }
    }

    /// Choose a weighted random item from possibilities
    fn weighted_choice(&self, choices: &[(T, f64)]) -> Option<T> {
        if choices.is_empty() {
            return None;
        }

        let total_weight: f64 = choices.iter().map(|(_, weight)| weight).sum();
        if total_weight <= 0.0 {
            return fastrand::choice(choices).map(|(item, _)| item.clone());
        }

        let mut random = fastrand::f64() * total_weight;

        for (item, weight) in choices {
            random -= weight;
            if random <= 0.0 {
                return Some(item.clone());
            }
        }

        // Fallback
        choices.last().map(|(item, _)| item.clone())
    }

    /// Try to find a partial match when exact state isn't found
    fn find_partial_match_and_continue(&self, result: &mut Vec<T>, current_state: &[T]) -> bool {
        // Try smaller context sizes
        for context_size in (1..self.order).rev() {
            if current_state.len() >= context_size {
                let partial_state = &current_state[current_state.len() - context_size..];

                // Look for any transition that ends with this partial state
                for (state, transitions) in &self.transitions {
                    if state.len() >= context_size
                        && &state[state.len() - context_size..] == partial_state
                        && let Some(next) = self.weighted_choice(transitions)
                    {
                        result.push(next);
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get statistics about the trained model
    pub fn stats(&self) -> MarkovStats {
        let total_states = self.transitions.len();
        let total_transitions: usize = self.transitions.values().map(|v| v.len()).sum();

        MarkovStats {
            order: self.order,
            total_states,
            total_transitions,
            starting_states_count: self.starting_states.len(),
        }
    }
}

/// Statistics about a trained Markov chain
#[derive(Debug, Clone)]
pub struct MarkovStats {
    pub order: usize,
    pub total_states: usize,
    pub total_transitions: usize,
    pub starting_states_count: usize,
}

/// Specialized Markov chain for melody generation
#[derive(Debug, Clone)]
pub struct MelodyMarkov {
    /// Internal Markov chain working with pitch intervals
    chain: MarkovChain<i8>,

    /// Whether to use absolute pitches or relative intervals
    use_intervals: bool,
}

impl MelodyMarkov {
    /// Create a new melody Markov chain
    pub fn new(order: usize, use_intervals: bool) -> Self {
        Self {
            chain: MarkovChain::new(order),
            use_intervals,
        }
    }

    /// Train on a chord progression, extracting melodies from chord tones
    pub fn train_from_chords(&mut self, chords: &[Chord]) {
        let sequences = chords
            .iter()
            .map(|chord| self.chord_to_sequence(chord))
            .collect::<Vec<_>>();

        let sequences_refs: Vec<&[i8]> = sequences.iter().map(|seq| seq.as_slice()).collect();

        self.chain.train_multiple(&sequences_refs);
    }

    /// Train on explicit note sequences
    pub fn train_from_notes(&mut self, note_sequences: &[&[Note]]) {
        for sequence in note_sequences {
            let converted = self.notes_to_training_sequence(sequence);
            self.chain.train(&converted);
        }
    }

    /// Generate a melody for a chord progression
    pub fn generate_melody(&self, chords: &[Chord], notes_per_chord: usize, octave: i8) -> Vec<Note> {
        let total_notes = chords.len() * notes_per_chord;

        if self.use_intervals {
            self.generate_interval_melody(chords, notes_per_chord, octave)
        } else {
            self.generate_absolute_melody(total_notes, octave)
        }
    }

    /// Convert a chord to a training sequence (chord tones as intervals)
    fn chord_to_sequence(&self, chord: &Chord) -> Vec<i8> {
        let chord_tones = chord.notes(4); // Use octave 4 for training

        if self.use_intervals {
            // Convert to intervals between chord tones
            let mut intervals = Vec::new();
            for window in chord_tones.windows(2) {
                let interval = window[1].to_midi() as i8 - window[0].to_midi() as i8;
                intervals.push(interval);
            }
            // Add return interval to complete the pattern
            if chord_tones.len() > 1
                && let Some(first) = chord_tones.first()
                && let Some(last) = chord_tones.last()
            {
                let return_interval = first.to_midi() as i8 - last.to_midi() as i8;
                intervals.push(return_interval);
            }
            intervals
        } else {
            // Use absolute pitch classes
            chord_tones.iter().map(|note| note.pitch_class as i8).collect()
        }
    }

    /// Convert note sequence to training data
    fn notes_to_training_sequence(&self, notes: &[Note]) -> Vec<i8> {
        if self.use_intervals {
            // Convert to intervals
            notes
                .windows(2)
                .map(|window| {
                    let diff = window[1].to_midi() as i8 - window[0].to_midi() as i8;
                    diff.clamp(-12, 12) // Limit to one octave
                })
                .collect()
        } else {
            // Use pitch classes
            notes.iter().map(|note| note.pitch_class as i8).collect()
        }
    }

    /// Generate melody using intervals
    fn generate_interval_melody(&self, chords: &[Chord], notes_per_chord: usize, octave: i8) -> Vec<Note> {
        let mut melody = Vec::new();

        // Start with root of first chord
        let mut current_midi = (octave + 4) * 12 + chords[0].root as i8;
        melody.push(Note::from_midi(current_midi as u8));

        for chord in chords {
            let chord_tones: Vec<i8> = chord.notes(4).iter().map(|note| note.pitch_class as i8).collect();

            for _ in 0..notes_per_chord {
                // Generate interval using Markov chain
                let intervals = self.chain.generate(2);

                if let Some(&interval) = intervals.first() {
                    current_midi += interval;

                    // Keep within reasonable range
                    current_midi = current_midi.clamp(24, 108); // C1 to C8

                    // Bias towards chord tones
                    let current_pc = current_midi % 12;
                    if !chord_tones.contains(&current_pc) && fastrand::f64() < 0.3 {
                        // Snap to a randomly chosen chord tone within the same octave
                        if let Some(&random_chord_tone) = fastrand::choice(&chord_tones) {
                            let target_midi = current_midi - current_pc + random_chord_tone;
                            current_midi = target_midi;
                        }
                    }

                    melody.push(Note::from_midi(current_midi as u8));
                }
            }
        }

        melody
    }

    /// Generate melody using absolute pitches
    fn generate_absolute_melody(&self, length: usize, octave: i8) -> Vec<Note> {
        let pitch_classes = self.chain.generate(length);

        pitch_classes
            .iter()
            .map(|&pc| {
                let midi = (octave + 4) * 12 + pc;
                Note::from_midi(midi.clamp(0, 127) as u8)
            })
            .collect()
    }

    /// Get training statistics
    pub fn stats(&self) -> MarkovStats {
        self.chain.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chord;

    #[test]
    fn markov_chain_basic_functionality() {
        let mut chain = MarkovChain::new(1);
        let sequence = vec![1, 2, 3, 1, 2, 3, 1, 2, 3];

        chain.train(&sequence);

        let stats = chain.stats();
        assert_eq!(stats.order, 1);
        assert!(stats.total_states > 0);
        assert!(stats.total_transitions > 0);
    }

    #[test]
    fn markov_chain_generation() {
        let mut chain = MarkovChain::new(1);
        let sequence = vec![1, 2, 3, 1, 2, 3, 1, 2, 3];

        chain.train(&sequence);

        let generated = chain.generate(6);

        assert_eq!(generated.len(), 6);
        // Should contain values from the original sequence
        for &value in &generated {
            assert!([1, 2, 3].contains(&value));
        }
    }

    #[test]
    fn melody_markov_chord_training() {
        let mut melody_markov = MelodyMarkov::new(2, true);

        let chords = vec![Chord::c_major(), Chord::f_major(), Chord::g_major()];

        melody_markov.train_from_chords(&chords);

        let stats = melody_markov.stats();
        assert_eq!(stats.order, 2);
        assert!(stats.total_states > 0);
    }

    #[test]
    fn melody_markov_generation() {
        let mut melody_markov = MelodyMarkov::new(1, true);

        let chords = vec![Chord::c_major(), Chord::a_minor()];

        melody_markov.train_from_chords(&chords);

        let melody = melody_markov.generate_melody(&chords, 4, 5);

        // Melody should have at least some notes generated
        assert!(!melody.is_empty());
        // Should be approximately the expected length (within reasonable bounds)
        let expected_length = chords.len() * 4;
        assert!(melody.len() >= expected_length - 2 && melody.len() <= expected_length + 2);
    }

    #[test]
    fn test_different_orders() {
        for order in 1..=3 {
            let mut chain = MarkovChain::new(order);
            let sequence = vec![1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4];

            chain.train(&sequence);

            let stats = chain.stats();
            assert_eq!(stats.order, order);
        }
    }

    #[test]
    fn test_partial_matching() {
        let mut chain = MarkovChain::new(3);
        let sequence = vec![1, 2, 3, 4, 5, 6, 7, 8];

        chain.train(&sequence);

        let generated = chain.generate(5);

        assert!(generated.len() <= 5);
    }
}
