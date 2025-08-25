use crate::harmony::TypedHarmonicPattern;
use crate::melody::{MelodyGenerator, MelodyStrategy};
use crate::midi::Composition;
use crate::rhythm::PatternLibrary;
use crate::{Chord, LobachevskyError, ProgressionBuilder, RhythmPattern};

pub struct AlgorithmicComposition {
    rhythm: Box<dyn RhythmPattern>,
    harmony: TypedHarmonicPattern,
    melody: MelodyStrategy,
    bars: usize,
    tempo: u16,
    notes_per_chord: usize,
}

impl AlgorithmicComposition {
    /// Create a new algorithmic composition with typed inputs
    pub fn new(
        harmony: TypedHarmonicPattern,
        rhythm: Box<dyn RhythmPattern>,
        melody: MelodyStrategy,
        bars: usize,
        tempo: u16,
        notes_per_chord: usize,
    ) -> Self {
        Self {
            rhythm,
            harmony,
            melody,
            bars,
            tempo,
            notes_per_chord,
        }
    }

    /// Load a rhythm pattern from a file
    pub fn load_rhythm_pattern(rhythm_file: &str) -> Result<Box<dyn RhythmPattern>, LobachevskyError> {
        println!("🥁 Loading rhythm pattern: {}", rhythm_file);
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
        println!("🎵 Generating Algorithmic Composition");
        println!("=====================================");

        // Align bars to 16 or 32 bar multiples
        let aligned_bars = if self.bars <= 16 {
            16
        } else if self.bars <= 32 {
            32
        } else {
            self.bars.div_ceil(16) // Round up to nearest 16
        };

        if aligned_bars != self.bars {
            println!(
                "📏 Aligning from {} to {} bars for better musical structure",
                self.bars, aligned_bars
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
            println!("🎵 Modal constraint: {} {}", tonic, mode);
        }

        // Force return to start if needed for alignment
        if self.harmony.return_to_start || (total_chords_needed % chords_per_pattern != 0) {
            builder = builder.with_return();
            println!("🔄 Returning to starting chord for structural alignment");
        }

        let progression = builder.build();

        // Trim or extend to exact bar count
        let final_progression: Vec<Chord> = progression.into_iter().cycle().take(aligned_bars).collect();

        println!(
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
            println!("   Generated {} drum events", rhythm_events.len());
        }

        // Generate melody
        println!("🎵 Generating melody with {} strategy", self.melody);
        let melody_gen = MelodyGenerator::new(self.melody.clone())
            .with_octave(5)
            .with_note_duration(0.25); // Quarter notes by default

        let melody = melody_gen.generate(&final_progression, self.notes_per_chord);
        println!("   Generated {} notes", melody.len());

        // Create composition
        println!("💿 Creating MIDI composition at {} BPM", self.tempo);
        let mut composition = Composition::new(self.tempo);

        // Add tracks
        composition.add_harmony_track(&final_progression, 4, 3); // Each chord for 1 bar, octave 3
        composition.add_melody_track(&melody, 1); // Channel 1 for melody

        if !rhythm_events.is_empty() {
            composition.add_rhythm_track_from_events(&rhythm_events);
        }

        // Save
        composition.save(output)?;
        println!("✅ Saved algorithmic composition to {}", output);
        println!("   {} bars at {} BPM", aligned_bars, self.tempo);
        Ok(())
    }
}

pub fn algorithmic_composition(
    harmony: TypedHarmonicPattern,
    rhythm: Box<dyn RhythmPattern>,
    melody: MelodyStrategy,
    bars: usize,
    tempo: u16,
    notes_per_chord: usize,
    output: &str,
) -> Result<(), LobachevskyError> {
    let composition = AlgorithmicComposition::new(harmony, rhythm, melody, bars, tempo, notes_per_chord);

    composition.generate(output)
}
