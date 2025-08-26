//! Implementation of the `generate` command, which pulls together all the
//! algorithms to generate an algorithmic composition.

use super::*;
use crate::bass::{BassGenerator, BassStrategy};
use crate::harmony::TypedHarmonicPattern;
use crate::library::{Library, PatternLibrary};
use crate::melody::{CallResponseGenerator, CallResponseType, MelodyGenerator, MelodyStrategy};
use crate::midi::Composition;
use crate::rhythm::EuclideanPattern;
use crate::{Chord, LobachevskyError, ProgressionBuilder, RhythmPattern};

/// Input for the Generate command
pub struct GenerateInput {
    pub rhythm_name: Option<String>,
    pub harmony: TypedHarmonicPattern,
    pub melody_strategy: MelodyStrategy,
    pub bass_strategy: Option<BassStrategy>,
    pub call_response_type: Option<CallResponseType>,
    pub notes_per_chord: usize,
}

impl GenerateInput {
    #[allow(clippy::too_many_arguments)]
    pub fn from_cli(
        rhythm: Option<&str>,
        harmony_file: Option<&str>,
        start: &str,
        pattern: &str,
        mode: Option<&str>,
        tonic: Option<&str>,
        melody: &str,
        bars: usize,
        tempo: u16,
        notes_per_chord: usize,
        return_to_start: bool,
    ) -> Result<Self, LobachevskyError> {
        // Load or create harmonic pattern
        let harmony = if let Some(harmony_path) = harmony_file {
            // Load from file
            if harmony_path.ends_with(".toml") {
                let mut lib = crate::library::HarmonicLibrary::new();
                let pattern = lib.load_from_file(std::path::Path::new(harmony_path))?;
                pattern.to_typed()?
            } else {
                // Assume it's a pattern name from the harmonics directory
                let mut lib = crate::library::HarmonicLibrary::new();
                lib.load_from_directory(std::path::Path::new(crate::library::HARMONICS_LIB))?;
                let pattern = lib.get(harmony_path).ok_or_else(|| LobachevskyError::ParseError {
                    message: format!("Harmonic pattern '{}' not found", harmony_path),
                })?;
                pattern.to_typed()?
            }
        } else {
            // Create from command line arguments
            let start_chord = Chord::try_from(start)?;
            let transforms = parse_transforms(pattern)?;
            let (parsed_mode, parsed_tonic) = parse_mode_and_tonic(mode, tonic)?;

            TypedHarmonicPattern {
                name: "cli_generated".to_string(),
                description: Some("Generated from command line arguments".to_string()),
                start_chord,
                transformations: transforms,
                mode: parsed_mode,
                tonic: parsed_tonic,
                return_to_start,
                tempo_hint: Some(tempo),
                bars_hint: Some(bars),
            }
        };

        let melody_strategy = MelodyStrategy::from(melody);

        Ok(GenerateInput {
            rhythm_name: rhythm.map(|s| s.to_string()),
            harmony,
            melody_strategy,
            bass_strategy: None,      // Can be added to CLI later
            call_response_type: None, // Can be added to CLI later
            notes_per_chord,
        })
    }
}

pub struct AlgorithmicComposition {
    rhythm: Box<dyn RhythmPattern>,
    harmony: TypedHarmonicPattern,
    melody: MelodyStrategy,
    bass_strategy: Option<BassStrategy>,
    call_response_type: Option<CallResponseType>,
    bars: usize,
    tempo: u16,
    notes_per_chord: usize,
}

impl AlgorithmicComposition {
    #[allow(clippy::too_many_arguments)]
    /// Create a new algorithmic composition with typed inputs
    pub fn new(
        harmony: TypedHarmonicPattern,
        rhythm: Box<dyn RhythmPattern>,
        melody: MelodyStrategy,
        bass_strategy: Option<BassStrategy>,
        call_response_type: Option<CallResponseType>,
        bars: usize,
        tempo: u16,
        notes_per_chord: usize,
    ) -> Self {
        Self {
            rhythm,
            harmony,
            melody,
            bass_strategy,
            call_response_type,
            bars,
            tempo,
            notes_per_chord,
        }
    }

    /// Load a rhythm pattern from a file
    pub fn load_rhythm_pattern(rhythm_file: &str) -> Result<Box<dyn RhythmPattern>, LobachevskyError> {
        log::info!("🥁 Loading rhythm pattern: {}", rhythm_file);
        let mut pattern_lib = PatternLibrary::new();
        pattern_lib.load_from_directory(std::path::Path::new("patterns"))?;

        let pattern_data = pattern_lib
            .get(rhythm_file)
            .ok_or_else(|| LobachevskyError::ParseError {
                message: format!("Rhythm pattern '{}' not found", rhythm_file),
            })?;

        pattern_data.to_pattern()
    }

    /// Generate the composition and save it in a MIDI file
    pub fn generate(&self, output: &str) -> Result<(), LobachevskyError> {
        log::info!("🎵 Generating Algorithmic Composition");
        log::info!("=====================================");

        // Align bars to 16 or 32 bar multiples
        let aligned_bars = calculate_aligned_bars(self.bars);

        if aligned_bars != self.bars {
            log::info!(
                "📏 Aligning from {} to {} bars for better musical structure",
                self.bars,
                aligned_bars
            );
        }

        // Build progression
        let starting_chord = self.harmony.start_chord;
        let transforms = self.harmony.transformations.clone();

        // Calculate how many times to repeat the pattern to fill the bars
        // Each chord gets 1 bar by default
        let chords_per_pattern = transforms.len() + 1; // +1 for starting chord
        let total_chords_needed = aligned_bars;
        let pattern_repetitions = total_chords_needed.div_ceil(chords_per_pattern);

        let mut builder = ProgressionBuilder::new()
            .start(starting_chord)
            .pattern(&transforms)
            .length(chords_per_pattern * pattern_repetitions);

        // Apply modal constraints if specified
        if let (Some(mode), Some(tonic)) = (self.harmony.mode, self.harmony.tonic) {
            builder = builder.with_mode(mode, tonic);
            log::info!("🎵 Modal constraint: {} {}", tonic, mode);
        }

        // Force return to start if needed for alignment
        if self.harmony.return_to_start || (total_chords_needed % chords_per_pattern != 0) {
            builder = builder.with_return();
            log::info!("🔄 Returning to starting chord for structural alignment");
        }

        let progression = builder.build();

        // Trim or extend to exact bar count
        let final_progression: Vec<Chord> = progression.into_iter().cycle().take(aligned_bars).collect();

        log::info!(
            "🎹 Generated {} chords for {} bars",
            final_progression.len(),
            aligned_bars
        );

        // Generate rhythm events for all bars
        let mut rhythm_events = Vec::new();
        for bar in 0..aligned_bars {
            let bar_events = self.rhythm.events_for_bar(bar);
            // Adjust timing to account for bar position
            for mut event in bar_events {
                event.beat = crate::rhythm::Beat(event.beat.0 + (bar as f64 * 4.0));
                rhythm_events.push(event);
            }
        }
        if !rhythm_events.is_empty() {
            log::info!("   Generated {} drum events", rhythm_events.len());
        }

        // Generate melody or call-and-response patterns
        let melody = if let Some(ref call_response_type) = self.call_response_type {
            log::info!(
                "🎤 Generating call-and-response patterns with {} type",
                call_response_type
            );
            let cr_gen = CallResponseGenerator::new(call_response_type.clone(), self.notes_per_chord)
                .with_note_duration(0.5)
                .with_octave_range(4, 6);
            let phrases = cr_gen.generate(&final_progression);
            log::info!("   Generated {} call-response phrases", phrases.len());

            // Flatten all notes from call-and-response phrases
            phrases.iter().flat_map(|phrase| phrase.all_notes()).collect()
        } else {
            log::info!("🎵 Generating melody with {} strategy", self.melody);
            let melody_gen = MelodyGenerator::new(self.melody.clone())
                .with_octave(5)
                .with_note_duration(0.25); // Quarter notes by default

            let melody = melody_gen.generate(&final_progression, self.notes_per_chord);
            log::info!("   Generated {} notes", melody.len());
            melody
        };

        // Generate bass line if strategy is specified
        let bass_line = if let Some(ref bass_strategy) = self.bass_strategy {
            log::info!("🎸 Generating bass line with {} strategy", bass_strategy);
            let bass_gen = BassGenerator::new(bass_strategy.clone())
                .with_octave(2)
                .with_note_duration(1.0); // Whole notes by default
            let bass = bass_gen.generate(&final_progression, 1); // One bass note per chord
            log::info!("   Generated {} bass notes", bass.len());
            Some(bass)
        } else {
            None
        };

        // Create composition
        log::info!("💿 Creating MIDI composition at {} BPM", self.tempo);
        let mut composition = Composition::new(self.tempo);

        // Add tracks
        composition.add_harmony_track(&final_progression, 4, 3); // Each chord for 1 bar, octave 3
        composition.add_melody_track(&melody, 1); // Channel 1 for melody

        if let Some(bass) = bass_line {
            composition.add_bass_track(&bass, 2); // Channel 2 for bass
        }

        if !rhythm_events.is_empty() {
            composition.add_rhythm_track_from_events(&rhythm_events);
        }

        // Save
        composition.save(output)?;
        log::info!("✅ Saved algorithmic composition to {}", output);
        log::info!("   {} bars at {} BPM", aligned_bars, self.tempo);
        Ok(())
    }
}

/// Calculate aligned bar count for musical structure
/// Rounds up to 16 bars minimum, 32 bars for medium lengths,
/// or the next multiple of 16 for longer pieces
fn calculate_aligned_bars(bars: usize) -> usize {
    if bars <= 16 {
        16
    } else if bars <= 32 {
        32
    } else {
        // Round up to next multiple of 16
        bars.div_ceil(16)
    }
}

pub fn generate_algorithmic(
    input: GenerateInput,
    bars: usize,
    tempo: u16,
    output: &str,
) -> Result<(), LobachevskyError> {
    // Load or create rhythm pattern
    let rhythm = if let Some(rhythm_name) = &input.rhythm_name {
        AlgorithmicComposition::load_rhythm_pattern(rhythm_name)?
    } else {
        Box::new(EuclideanPattern::default())
    };

    let composition = AlgorithmicComposition::new(
        input.harmony, rhythm, input.melody_strategy, input.bass_strategy, input.call_response_type, bars, tempo,
        input.notes_per_chord,
    );

    composition.generate(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_alignment() {
        // Test cases: (input, expected_output)
        let test_cases = vec![
            (8, 16),    // Small number → 16
            (16, 16),   // Exactly 16 → 16
            (17, 32),   // Just over 16 → 32
            (32, 32),   // Exactly 32 → 32
            (33, 48),   // Just over 32 → 48 (next multiple of 16)
            (48, 48),   // Multiple of 16 → stays same
            (64, 64),   // Multiple of 16 → stays same
            (100, 112), // Non-multiple → round up to 112
            (127, 128), // Almost 128 → round up to 128
            (128, 128), // Exactly 128 → stays 128
            (200, 208), // Large non-multiple → round up to 208
            (256, 256), // Large multiple → stays same
        ];

        for (input, expected) in test_cases {
            let aligned = calculate_aligned_bars(input);
            assert_eq!(
                aligned, expected,
                "Bar alignment failed: {} bars should align to {} but got {}",
                input, expected, aligned
            );
        }
    }

    #[test]
    fn test_bar_alignment_edge_cases() {
        assert_eq!(calculate_aligned_bars(0), 16); // Zero bars → 16
        assert_eq!(calculate_aligned_bars(1), 16); // One bar → 16
        assert_eq!(calculate_aligned_bars(15), 16); // 15 bars → 16
        assert_eq!(calculate_aligned_bars(31), 32); // 31 bars → 32
        assert_eq!(calculate_aligned_bars(1000), 1008); // Large number → next multiple of 16
    }
}
