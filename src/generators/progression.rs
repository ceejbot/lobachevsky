//! Implementation of the progression generator.

use super::*;
use crate::midi::Composition;
use crate::{Chord, ProgressionBuilder};

/// Input for the Progression command
pub struct ProgressionInput {
    pub start_chord: Chord,
    pub transforms: Vec<Transform>,
    pub length: usize,
    pub return_to_start: bool,
    pub mode: Option<Mode>,
    pub tonic: Option<PitchClass>,
}

impl ProgressionInput {
    pub fn from_cli(
        start: &str,
        pattern: &str,
        length: usize,
        return_to_start: bool,
        mode: Option<&str>,
        tonic: Option<&str>,
    ) -> Result<Self, LobachevskyError> {
        let start_chord = Chord::try_from(start)?;
        let transforms = parse_transforms(pattern)?;

        if transforms.is_empty() {
            return Err(LobachevskyError::ParseError {
                message: "Invalid transformation pattern: no transforms parsed".to_string(),
            });
        }

        let (parsed_mode, parsed_tonic) = parse_mode_and_tonic(mode, tonic)?;

        Ok(ProgressionInput {
            start_chord,
            transforms,
            length,
            return_to_start,
            mode: parsed_mode,
            tonic: parsed_tonic,
        })
    }
}

pub fn generate_progression(input: progression::ProgressionInput, output: &str) -> Result<(), LobachevskyError> {
    let mut builder = ProgressionBuilder::new()
        .start(input.start_chord)
        .pattern(&input.transforms)
        .length(input.length);

    if input.return_to_start {
        builder = builder.with_return();
    }

    // Apply modal constraints if specified
    if let (Some(mode), Some(tonic)) = (input.mode, input.tonic) {
        builder = builder.with_mode(mode, tonic);
        println!("Using modal constraint: {} {}", tonic, mode);
    }

    let progression = builder.build();

    // Print the progression
    println!("Generated progression:");
    for (i, chord) in progression.iter().enumerate() {
        println!("  {}: {}", i + 1, chord);
    }

    // Generate MIDI
    let mut composition = Composition::new(110);
    composition.add_harmony_track(&progression, 4, 4);

    composition.save(output)?;
    println!("Saved to {}", output);
    Ok(())
}
