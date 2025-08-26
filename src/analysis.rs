//! Musical analysis utilities
//!
//! This module provides tools for analyzing harmonic content, measuring energy
//! levels, and extracting musical features that can drive adaptive pattern
//! generation.

use crate::{Chord, Note, PitchClass};

/// Measures different aspects of harmonic and melodic energy
#[derive(Debug, Clone)]
pub struct HarmonicAnalyzer {
    tonic: Option<PitchClass>,
}

impl Default for HarmonicAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl HarmonicAnalyzer {
    pub fn new() -> Self {
        Self { tonic: None }
    }

    /// Set a tonic for modal analysis
    pub fn with_tonic(mut self, tonic: PitchClass) -> Self {
        self.tonic = Some(tonic);
        self
    }

    /// Calculate the harmonic energy of a chord progression
    /// Returns a value from 0.0 (low energy) to 1.0 (high energy)
    pub fn analyze_progression_energy(&self, chords: &[Chord]) -> f64 {
        if chords.is_empty() {
            return 0.0;
        }

        let voice_leading_energy = self.voice_leading_energy(chords);
        let chord_complexity_energy = self.chord_complexity_energy(chords);
        let harmonic_rhythm_energy = self.harmonic_rhythm_energy(chords);
        let modal_tension_energy = self.modal_tension_energy(chords);

        // Weighted combination of different energy measures
        (voice_leading_energy * 0.3
            + chord_complexity_energy * 0.2
            + harmonic_rhythm_energy * 0.2
            + modal_tension_energy * 0.3)
            .clamp(0.0, 1.0)
    }

    /// Calculate the energy of a specific chord in context
    pub fn analyze_chord_energy(&self, chord: Chord, previous: Option<Chord>) -> f64 {
        let complexity = self.single_chord_complexity(chord);
        let voice_leading = if let Some(prev) = previous {
            self.chord_transition_energy(prev, chord)
        } else {
            0.0
        };
        let modal_tension = if let Some(tonic) = self.tonic {
            self.chord_modal_tension(chord, tonic)
        } else {
            0.0
        };

        (complexity * 0.4 + voice_leading * 0.4 + modal_tension * 0.2).clamp(0.0, 1.0)
    }

    /// Analyze the energy of a melodic line
    pub fn analyze_melody_energy(&self, notes: &[(Note, f64, f64)]) -> f64 {
        if notes.len() < 2 {
            return 0.0;
        }

        let interval_energy = self.interval_energy(notes);
        let rhythmic_energy = self.rhythmic_density_energy(notes);
        let range_energy = self.pitch_range_energy(notes);

        (interval_energy * 0.4 + rhythmic_energy * 0.3 + range_energy * 0.3).clamp(0.0, 1.0)
    }

    /// Voice leading energy - measures smoothness vs. large jumps
    fn voice_leading_energy(&self, chords: &[Chord]) -> f64 {
        if chords.len() < 2 {
            return 0.0;
        }

        let mut total_distance = 0.0;
        let mut transitions = 0;

        for window in chords.windows(2) {
            let distance = self.chord_transition_energy(window[0], window[1]);
            total_distance += distance;
            transitions += 1;
        }

        if transitions == 0 {
            0.0
        } else {
            total_distance / transitions as f64
        }
    }

    /// Energy from chord complexity (extensions, alterations)
    fn chord_complexity_energy(&self, chords: &[Chord]) -> f64 {
        let total_complexity: f64 = chords.iter().map(|&chord| self.single_chord_complexity(chord)).sum();
        total_complexity / chords.len() as f64
    }

    /// Energy from harmonic rhythm (how often chords change)
    fn harmonic_rhythm_energy(&self, chords: &[Chord]) -> f64 {
        // For now, assume each chord lasts one beat
        // More frequent changes = higher energy
        let changes_per_bar = chords.len() as f64 / (chords.len() as f64 / 4.0).max(1.0);
        (changes_per_bar / 4.0).clamp(0.0, 1.0) // Normalize assuming max 4 changes per bar
    }

    /// Energy from modal tension relative to tonic
    fn modal_tension_energy(&self, chords: &[Chord]) -> f64 {
        if let Some(tonic) = self.tonic {
            let total_tension: f64 = chords.iter().map(|&chord| self.chord_modal_tension(chord, tonic)).sum();
            total_tension / chords.len() as f64
        } else {
            0.0
        }
    }

    /// Calculate transition energy between two chords
    fn chord_transition_energy(&self, from: Chord, to: Chord) -> f64 {
        let from_notes = from.notes(4);
        let to_notes = to.notes(4);

        // Calculate minimum voice leading distance
        let mut total_distance = 0.0;
        let min_voices = from_notes.len().min(to_notes.len());

        for i in 0..min_voices {
            let distance = (to_notes[i].to_midi() as i16 - from_notes[i].to_midi() as i16).abs();
            total_distance += distance as f64;
        }

        // Add penalty for different chord sizes
        let size_penalty = (from_notes.len() as i16 - to_notes.len() as i16).abs() as f64 * 2.0;
        total_distance += size_penalty;

        // Normalize - large jumps (>12 semitones) are high energy
        (total_distance / (min_voices as f64 * 12.0)).clamp(0.0, 1.0)
    }

    /// Calculate complexity of a single chord
    fn single_chord_complexity(&self, chord: Chord) -> f64 {
        // Basic triad = low complexity, extended chords = higher complexity
        let notes = chord.notes(4);
        let base_complexity = match notes.len() {
            3 => 0.2, // Triad
            4 => 0.5, // Seventh
            5 => 0.8, // Ninth/extended
            _ => 1.0, // Complex/altered
        };

        // Add complexity for specific chord qualities
        let quality_bonus = match chord.quality {
            crate::ChordQuality::Major => 0.0,
            crate::ChordQuality::Minor => 0.1,
            crate::ChordQuality::Diminished => 0.3,
            crate::ChordQuality::Augmented => 0.4,
            _ => 0.2, // Other qualities
        };

        let result: f64 = base_complexity + quality_bonus;
        result.clamp(0.0, 1.0)
    }

    /// Calculate modal tension of a chord relative to tonic
    fn chord_modal_tension(&self, chord: Chord, tonic: PitchClass) -> f64 {
        let root_interval = ((chord.root.to_semitone() as i16 - tonic.to_semitone() as i16 + 12) % 12) as u8;

        // Circle of fifths distance creates tension
        // Perfect consonances (1, 5, 8) are low tension
        // Tritone and leading tones are high tension
        match root_interval {
            0 => 0.0,             // Tonic - no tension
            5 | 7 => 0.2,         // Perfect 4th/5th - low tension
            3 | 4 | 8 | 9 => 0.6, // Major/minor 3rd, 6th - medium tension
            6 => 1.0,             // Tritone - maximum tension
            1 | 11 => 0.8,        // Minor 2nd, major 7th - high tension
            2 | 10 => 0.4,        // Major 2nd, minor 7th - medium tension
            _ => 0.5,             // Default
        }
    }

    /// Calculate interval energy in a melody
    fn interval_energy(&self, notes: &[(Note, f64, f64)]) -> f64 {
        if notes.len() < 2 {
            return 0.0;
        }

        let mut total_energy = 0.0;
        for window in notes.windows(2) {
            let interval = (window[1].0.to_midi() as i16 - window[0].0.to_midi() as i16).abs();
            // Larger intervals = higher energy
            total_energy += (interval as f64 / 12.0).clamp(0.0, 1.0);
        }

        total_energy / (notes.len() - 1) as f64
    }

    /// Calculate rhythmic density energy
    fn rhythmic_density_energy(&self, notes: &[(Note, f64, f64)]) -> f64 {
        if notes.is_empty() {
            return 0.0;
        }

        // Calculate notes per beat
        let Some(last) = notes.last() else {
            return 0.0;
        };
        let total_duration = last.1 - notes[0].1 + last.2;
        let notes_per_beat = notes.len() as f64 / total_duration.max(1.0);

        // Normalize assuming 4 notes per beat is maximum density
        (notes_per_beat / 4.0).clamp(0.0, 1.0)
    }

    /// Calculate pitch range energy
    fn pitch_range_energy(&self, notes: &[(Note, f64, f64)]) -> f64 {
        if notes.is_empty() {
            return 0.0;
        }

        let midi_notes: Vec<u8> = notes.iter().map(|(note, _, _)| note.to_midi()).collect();
        let min_pitch = *midi_notes.iter().min().unwrap_or(&0);
        let max_pitch = *midi_notes.iter().max().unwrap_or(&48);
        let range = max_pitch - min_pitch;

        // Normalize assuming 2 octaves (24 semitones) is maximum range
        (range as f64 / 24.0).clamp(0.0, 1.0)
    }
}

/// Energy levels for driving adaptive patterns
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnergyLevel {
    Low,    // 0.0 - 0.33
    Medium, // 0.33 - 0.66
    High,   // 0.66 - 1.0
}

impl From<f64> for EnergyLevel {
    fn from(energy: f64) -> Self {
        if energy < 0.33 {
            EnergyLevel::Low
        } else if energy < 0.66 {
            EnergyLevel::Medium
        } else {
            EnergyLevel::High
        }
    }
}

impl EnergyLevel {
    /// Convert energy level to a multiplier for pattern intensity
    pub fn intensity_multiplier(&self) -> f64 {
        match self {
            EnergyLevel::Low => 0.5,
            EnergyLevel::Medium => 1.0,
            EnergyLevel::High => 1.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chord;

    #[test]
    fn harmonic_analyzer_basic_energy() {
        let analyzer = HarmonicAnalyzer::new();
        let chords = vec![Chord::c_major(), Chord::f_major(), Chord::g_major()];

        let energy = analyzer.analyze_progression_energy(&chords);
        assert!((0.0..=1.0).contains(&energy));
    }

    #[test]
    fn energy_level_conversion() {
        assert_eq!(EnergyLevel::from(0.1), EnergyLevel::Low);
        assert_eq!(EnergyLevel::from(0.5), EnergyLevel::Medium);
        assert_eq!(EnergyLevel::from(0.8), EnergyLevel::High);
    }

    #[test]
    fn chord_complexity_analysis() {
        let analyzer = HarmonicAnalyzer::new();

        let simple_energy = analyzer.single_chord_complexity(Chord::c_major());
        let complex_chord = Chord::new(crate::PitchClass::C, crate::ChordQuality::Diminished);
        let complex_energy = analyzer.single_chord_complexity(complex_chord);

        assert!(complex_energy > simple_energy);
    }

    #[test]
    fn voice_leading_analysis() {
        let analyzer = HarmonicAnalyzer::new();

        // Smooth progression (C - F - G)
        let smooth_progression = vec![Chord::c_major(), Chord::f_major(), Chord::g_major()];
        let smooth_energy = analyzer.analyze_progression_energy(&smooth_progression);

        // More distant progression
        let distant_progression = vec![
            Chord::c_major(),
            Chord::new(crate::PitchClass::Fs, crate::ChordQuality::Major),
        ];
        let distant_energy = analyzer.analyze_progression_energy(&distant_progression);

        // Distant progression should have higher energy
        assert!(distant_energy >= smooth_energy);
    }
}
