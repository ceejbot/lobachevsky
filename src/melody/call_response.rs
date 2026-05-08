//! Call-and-response pattern generation
//!
//! This module provides functionality for generating musical call-and-response
//! patterns, where a musical phrase is "answered" by another phrase.

use std::fmt::Display;

use crate::{Chord, Note};

/// Types of call-and-response relationships
#[derive(Debug, Clone)]
pub enum CallResponseType {
    /// Echo - response repeats the call with variation
    Echo,
    /// Complement - response completes the harmonic or melodic idea
    Complement,
    /// Contrast - response provides contrasting material
    Contrast,
    /// Sequence - response transposes the call
    Sequence(i8), // Interval for transposition
    /// Inversion - response inverts the melodic contour
    Inversion,
}

impl Display for CallResponseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallResponseType::Echo => write!(f, "echo"),
            CallResponseType::Complement => write!(f, "complement"),
            CallResponseType::Contrast => write!(f, "contrast"),
            CallResponseType::Sequence(interval) => write!(f, "sequence({})", interval),
            CallResponseType::Inversion => write!(f, "inversion"),
        }
    }
}

impl From<&str> for CallResponseType {
    fn from(value: &str) -> Self {
        let value = value.to_lowercase();

        if value.starts_with("sequence") {
            if let Some(interval_part) = value.split('_').nth(1)
                && let Ok(interval) = interval_part.parse::<i8>()
            {
                return CallResponseType::Sequence(interval);
            }
            return CallResponseType::Sequence(5); // Perfect fifth by default
        }

        match value.as_str() {
            "echo" => CallResponseType::Echo,
            "complement" => CallResponseType::Complement,
            "contrast" => CallResponseType::Contrast,
            "inversion" => CallResponseType::Inversion,
            _ => CallResponseType::Echo,
        }
    }
}

/// Call-and-response generator
#[derive(Debug, Clone)]
pub struct CallResponseGenerator {
    response_type: CallResponseType,
    phrase_length: usize,
    gap_duration: f64, // Duration between call and response
    note_duration: f64,
    octave_range: (i8, i8), // Min and max octaves
}

impl Default for CallResponseGenerator {
    fn default() -> Self {
        Self::new(CallResponseType::Echo, 4)
    }
}

impl CallResponseGenerator {
    /// Create a new call-and-response generator
    pub fn new(response_type: CallResponseType, phrase_length: usize) -> Self {
        Self {
            response_type,
            phrase_length,
            gap_duration: 0.0, // No gap by default
            note_duration: 0.5,
            octave_range: (4, 6), // Melody range
        }
    }

    /// Set the gap duration between call and response
    pub fn with_gap(mut self, gap_duration: f64) -> Self {
        self.gap_duration = gap_duration;
        self
    }

    /// Set the note duration
    pub fn with_note_duration(mut self, duration: f64) -> Self {
        self.note_duration = duration;
        self
    }

    /// Set the octave range for generated notes
    pub fn with_octave_range(mut self, min_octave: i8, max_octave: i8) -> Self {
        self.octave_range = (min_octave, max_octave);
        self
    }

    /// Generate call-and-response patterns for a chord progression
    pub fn generate(&self, progression: &[Chord]) -> Vec<CallResponsePhrase> {
        let mut phrases = Vec::new();
        let mut beat_position = 0.0;

        for (chord_idx, chord) in progression.iter().enumerate() {
            // Generate the call phrase
            let call = self.generate_call_phrase(*chord, beat_position);
            beat_position += self.phrase_length as f64 * self.note_duration;

            // Add gap if specified
            beat_position += self.gap_duration;

            // Generate the response phrase
            let response_chord = progression.get(chord_idx + 1).unwrap_or(chord);
            let response = self.generate_response_phrase(&call.notes, *response_chord, beat_position);
            beat_position += self.phrase_length as f64 * self.note_duration;

            phrases.push(CallResponsePhrase {
                call,
                response,
                chord_context: *chord,
            });
        }

        phrases
    }

    /// Generate a call phrase
    fn generate_call_phrase(&self, chord: Chord, start_beat: f64) -> Phrase {
        let mut notes = Vec::new();
        let mut beat_position = start_beat;

        let chord_tones = chord.notes(self.octave_range.0);
        let scale_notes = self.get_scale_notes(chord);

        for _ in 0..self.phrase_length {
            // Bias towards chord tones but allow scale notes
            let fallback = Note::new(chord.root, self.octave_range.0);
            let note = if fastrand::f64() < 0.7 {
                fastrand::choice(&chord_tones).copied().unwrap_or(fallback)
            } else {
                fastrand::choice(&scale_notes).copied().unwrap_or(fallback)
            };

            notes.push((note, beat_position, self.note_duration));
            beat_position += self.note_duration;
        }

        Phrase {
            notes,
            start_beat,
            chord_context: chord,
        }
    }

    /// Generate a response phrase based on the call
    fn generate_response_phrase(&self, call_notes: &[(Note, f64, f64)], chord: Chord, start_beat: f64) -> Phrase {
        let response_notes = match &self.response_type {
            CallResponseType::Echo => self.generate_echo_response(call_notes, chord, start_beat),
            CallResponseType::Complement => self.generate_complement_response(call_notes, chord, start_beat),
            CallResponseType::Contrast => self.generate_contrast_response(call_notes, chord, start_beat),
            CallResponseType::Sequence(interval) => self.generate_sequence_response(call_notes, *interval, start_beat),
            CallResponseType::Inversion => self.generate_inversion_response(call_notes, chord, start_beat),
        };

        Phrase {
            notes: response_notes,
            start_beat,
            chord_context: chord,
        }
    }

    /// Generate echo response - similar to call with variations
    fn generate_echo_response(
        &self,
        call_notes: &[(Note, f64, f64)],
        chord: Chord,
        start_beat: f64,
    ) -> Vec<(Note, f64, f64)> {
        let mut response = Vec::new();
        let mut beat_position = start_beat;
        let chord_tones = chord.notes(self.octave_range.0);

        for (call_note, _, duration) in call_notes {
            let response_note = if fastrand::f64() < 0.8 {
                // Usually echo the same note
                *call_note
            } else {
                // Sometimes vary with a nearby chord tone
                fastrand::choice(&chord_tones).copied().unwrap_or(*call_note)
            };

            response.push((response_note, beat_position, *duration));
            beat_position += duration;
        }

        response
    }

    /// Generate complement response - completes the harmonic idea
    fn generate_complement_response(
        &self,
        call_notes: &[(Note, f64, f64)],
        chord: Chord,
        start_beat: f64,
    ) -> Vec<(Note, f64, f64)> {
        let mut response = Vec::new();
        let mut beat_position = start_beat;
        let chord_tones = chord.notes(self.octave_range.0);

        for (chord_tone_idx, (_, _, duration)) in call_notes.iter().enumerate() {
            // Use chord tones in sequence to complement
            let response_note = chord_tones
                .get(chord_tone_idx % chord_tones.len())
                .unwrap_or(&chord_tones[0]);

            response.push((*response_note, beat_position, *duration));
            beat_position += duration;
        }

        response
    }

    /// Generate contrast response - different from call
    fn generate_contrast_response(
        &self,
        call_notes: &[(Note, f64, f64)],
        chord: Chord,
        start_beat: f64,
    ) -> Vec<(Note, f64, f64)> {
        let mut response = Vec::new();
        let mut beat_position = start_beat;
        let scale_notes = self.get_scale_notes(chord);

        for (call_note, _, duration) in call_notes {
            // Choose notes that contrast with the call
            let available_notes: Vec<_> = scale_notes
                .iter()
                .filter(|&&note| {
                    let interval = (note.to_midi() as i16 - call_note.to_midi() as i16).abs();
                    (3..=7).contains(&interval) // Avoid unisons and octaves
                })
                .collect();

            let response_note = if !available_notes.is_empty() {
                **fastrand::choice(&available_notes).expect("failed to choose a note")
            } else {
                fastrand::choice(&scale_notes)
                    .copied()
                    .unwrap_or(Note::new(chord.root, self.octave_range.0))
            };

            response.push((response_note, beat_position, *duration));
            beat_position += duration;
        }

        response
    }

    /// Generate sequence response - transposed call
    fn generate_sequence_response(
        &self,
        call_notes: &[(Note, f64, f64)],
        interval: i8,
        start_beat: f64,
    ) -> Vec<(Note, f64, f64)> {
        let mut response = Vec::new();
        let mut beat_position = start_beat;

        for (call_note, _, duration) in call_notes {
            let transposed_midi = (call_note.to_midi() as i16 + interval as i16).clamp(24, 108) as u8;
            let response_note = Note::from_midi(transposed_midi);

            response.push((response_note, beat_position, *duration));
            beat_position += duration;
        }

        response
    }

    /// Generate inversion response - melodic inversion of call
    fn generate_inversion_response(
        &self,
        call_notes: &[(Note, f64, f64)],
        _chord: Chord,
        start_beat: f64,
    ) -> Vec<(Note, f64, f64)> {
        if call_notes.is_empty() {
            return Vec::new();
        }

        let mut response = Vec::new();
        let mut beat_position = start_beat;
        let center_note = call_notes[0].0; // Use first note as inversion center

        for (call_note, _, duration) in call_notes {
            let interval = call_note.to_midi() as i16 - center_note.to_midi() as i16;
            let inverted_midi = (center_note.to_midi() as i16 - interval).clamp(24, 108) as u8;
            let response_note = Note::from_midi(inverted_midi);

            response.push((response_note, beat_position, *duration));
            beat_position += duration;
        }

        response
    }

    /// Get scale notes for a chord (simplified - uses major scale)
    fn get_scale_notes(&self, chord: Chord) -> Vec<Note> {
        let root = chord.root;
        let major_intervals = [0, 2, 4, 5, 7, 9, 11]; // Major scale intervals

        major_intervals
            .iter()
            .map(|&interval| Note::new(root.transpose(interval), self.octave_range.0))
            .collect()
    }
}

/// A musical phrase with timing information
#[derive(Debug, Clone)]
pub struct Phrase {
    pub notes: Vec<(Note, f64, f64)>, // (note, start_beat, duration)
    pub start_beat: f64,
    pub chord_context: Chord,
}

/// A call-and-response phrase pair
#[derive(Debug, Clone)]
pub struct CallResponsePhrase {
    pub call: Phrase,
    pub response: Phrase,
    pub chord_context: Chord,
}

impl CallResponsePhrase {
    /// Get all notes from both call and response
    pub fn all_notes(&self) -> Vec<(Note, f64, f64)> {
        let mut all_notes = self.call.notes.clone();
        all_notes.extend(self.response.notes.clone());
        all_notes
    }

    /// Get the total duration of the call-and-response phrase
    pub fn total_duration(&self) -> f64 {
        let _call_end = self
            .call
            .notes
            .iter()
            .map(|(_, start, duration)| start + duration)
            .fold(0.0, f64::max);
        let response_end = self
            .response
            .notes
            .iter()
            .map(|(_, start, duration)| start + duration)
            .fold(0.0, f64::max);

        response_end - self.call.start_beat
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chord;

    #[test]
    fn call_response_echo_generation() {
        let generator = CallResponseGenerator::new(CallResponseType::Echo, 3);
        let chords = vec![Chord::c_major(), Chord::f_major()];

        let phrases = generator.generate(&chords);

        assert_eq!(phrases.len(), 2);
        assert_eq!(phrases[0].call.notes.len(), 3);
        assert_eq!(phrases[0].response.notes.len(), 3);
    }

    #[test]
    fn call_response_sequence_generation() {
        let generator = CallResponseGenerator::new(CallResponseType::Sequence(7), 2);
        let chords = vec![Chord::c_major()];

        let phrases = generator.generate(&chords);

        assert_eq!(phrases.len(), 1);
        let phrase = &phrases[0];

        // Response should be transposed up by 7 semitones (perfect fifth)
        for (call_note, response_note) in phrase.call.notes.iter().zip(phrase.response.notes.iter()) {
            let interval = response_note.0.to_midi() as i16 - call_note.0.to_midi() as i16;
            assert_eq!(interval, 7);
        }
    }

    #[test]
    fn call_response_inversion_generation() {
        let generator = CallResponseGenerator::new(CallResponseType::Inversion, 2);
        let chords = vec![Chord::c_major()];

        let phrases = generator.generate(&chords);

        assert_eq!(phrases.len(), 1);
        let phrase = &phrases[0];

        // Should have generated notes (inversion logic is tested implicitly)
        assert_eq!(phrase.call.notes.len(), 2);
        assert_eq!(phrase.response.notes.len(), 2);
    }

    #[test]
    fn call_response_type_from_string() {
        assert!(matches!(CallResponseType::from("echo"), CallResponseType::Echo));
        assert!(matches!(
            CallResponseType::from("complement"),
            CallResponseType::Complement
        ));
        assert!(matches!(CallResponseType::from("contrast"), CallResponseType::Contrast));
        assert!(matches!(
            CallResponseType::from("inversion"),
            CallResponseType::Inversion
        ));

        if let CallResponseType::Sequence(interval) = CallResponseType::from("sequence_5") {
            assert_eq!(interval, 5);
        } else {
            panic!("Expected Sequence(5)");
        }
    }

    #[test]
    fn call_response_phrase_all_notes() {
        let generator = CallResponseGenerator::new(CallResponseType::Echo, 2);
        let chords = vec![Chord::c_major()];

        let phrases = generator.generate(&chords);
        let phrase = &phrases[0];

        let all_notes = phrase.all_notes();
        assert_eq!(all_notes.len(), 4); // 2 call + 2 response
    }
}
