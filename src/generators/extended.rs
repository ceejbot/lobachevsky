//! The extended generate command.

use super::*;
use crate::Chord;
use crate::midi::Composition;

/// Input for the Extended command
pub struct ExtendedInput {
    pub start_chord: Chord,
    pub transforms: Vec<Transform>,
    pub length: usize,
    pub analyze: bool,
}

impl ExtendedInput {
    pub fn from_cli(start: &str, pattern: &str, length: usize, analyze: bool) -> Result<Self, LobachevskyError> {
        let start_chord = Chord::try_from(start)?;
        let transforms = parse_transforms(pattern)?;

        if transforms.is_empty() {
            return Err(LobachevskyError::ParseError {
                message: "Invalid transformation pattern: no transforms parsed".to_string(),
            });
        }

        Ok(ExtendedInput {
            start_chord,
            transforms,
            length,
            analyze,
        })
    }
}

pub fn generate_extended(input: ExtendedInput, output: &str) -> Result<(), LobachevskyError> {
    use crate::theory::{NeoRiemannian, Transformable};

    println!("Exploring extended chord transformations");

    if input.analyze {
        println!("\n=== Extended Chord Analysis ===");
        println!("Starting chord: {}", input.start_chord);
        println!("Chord type: {:?}", input.start_chord.quality);
        println!("Is triad: {}", input.start_chord.quality.is_triad());
        println!("Is seventh chord: {}", input.start_chord.quality.is_seventh());
        println!("Is suspended: {}", input.start_chord.quality.is_suspended());

        if let Some(underlying) = input.start_chord.underlying_triad() {
            println!("Underlying triad: {}", underlying);
        }

        println!(
            "Supports basic transforms (P, R, L): {}",
            input.start_chord.supports_basic_transforms()
        );
        println!(
            "Supports P3,0 transforms: {}",
            input.start_chord.supports_p3_transforms()
        );
    }

    // Generate extended chord progression
    let transformer = NeoRiemannian::new();
    let progression = transformer.apply_sequence_any(
        input.start_chord,
        &input.transforms[0..input.length.min(input.transforms.len())],
    );

    println!(
        "\nExtended chord progression ({} transformations):",
        progression.len() - 1
    );
    for (i, chord) in progression.iter().enumerate() {
        if i == 0 {
            println!("  Start: {} ({:?})", chord, chord.quality);
        } else {
            let transform = &input.transforms[(i - 1) % input.transforms.len()];
            let transform_name = match transform {
                Transform::P => "P",
                Transform::R => "R",
                Transform::L => "L",
                Transform::Compound(_) => "compound",
            };

            if input.analyze {
                println!("  {}: {} ({:?}) (via {})", i, chord, chord.quality, transform_name);
                if let Some(underlying) = chord.underlying_triad() {
                    println!("      Underlying: {}", underlying);
                }
            } else {
                println!("  {}: {} (via {})", i, chord, transform_name);
            }
        }
    }

    // Generate MIDI
    let mut composition = Composition::new(110);
    composition.add_harmony_track(&progression, 4, 2);

    composition.save(output)?;
    println!("\nSaved extended chord exploration to {}", output);
    Ok(())
}
