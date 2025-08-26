//! Implementation for the modal command.

use super::*;
use crate::midi::Composition;
use crate::{Chord, ModalNeoRiemannian};

/// Input for the Modal command
pub struct ModalInput {
    pub mode: Mode,
    pub tonic: PitchClass,
    pub start_chord: Option<Chord>,
    pub transforms: Vec<Transform>,
    pub length: usize,
    pub analyze: bool,
}

impl ModalInput {
    pub fn from_cli(
        mode: &str,
        tonic: &str,
        start: Option<&str>,
        pattern: &str,
        length: usize,
        analyze: bool,
    ) -> Result<Self, LobachevskyError> {
        let mode = Mode::try_from(mode)?;
        let tonic = PitchClass::try_from(tonic)?;
        let transforms = parse_transforms(pattern)?;

        if transforms.is_empty() {
            return Err(LobachevskyError::ParseError {
                message: "Invalid transformation pattern: no transforms parsed".to_string(),
            });
        }

        let start_chord = if let Some(start_str) = start {
            Some(Chord::try_from(start_str)?)
        } else {
            None
        };

        Ok(ModalInput {
            mode,
            tonic,
            start_chord,
            transforms,
            length,
            analyze,
        })
    }
}

pub fn generate_modal(input: modal::ModalInput, output: &str) -> Result<(), LobachevskyError> {
    println!(
        "Exploring {} {} mode with neo-Riemannian transformations",
        input.tonic, input.mode
    );

    // Create modal transformer
    let modal_transformer = ModalNeoRiemannian::new(input.mode, input.tonic);

    if input.analyze {
        let analysis = modal_transformer.analyze_mode();
        println!("\n=== Modal Analysis ===");
        println!("Mode: {} {}", input.tonic, input.mode);
        println!("Available chords:");
        for (i, chord) in analysis.chords.iter().enumerate() {
            let degree = match i {
                0 => "I",
                1 => "ii",
                2 => "iii",
                3 => "IV",
                4 => "V",
                5 => "vi",
                6 => "vii°",
                _ => "?",
            };
            println!("  {}: {}", degree, chord);
        }

        if !analysis.characteristic_chords.is_empty() {
            println!("Characteristic chords:");
            for chord in &analysis.characteristic_chords {
                println!("  {}", chord);
            }
        }
    }

    // Determine starting chord
    let start_chord = if let Some(chord) = input.start_chord {
        if modal_transformer.valid_chords().contains(&chord) {
            chord
        } else {
            eprintln!(
                "Warning: {} is not in {} {} mode, using first chord of mode",
                chord, input.tonic, input.mode
            );
            modal_transformer
                .valid_chords()
                .iter()
                .next()
                .copied()
                .unwrap_or(Chord::c_major())
        }
    } else {
        // Use the tonic chord of the mode
        input.mode.triads(input.tonic)[0]
    };

    println!("\nStarting from: {}", start_chord);

    // Generate modal progression using the transformer directly
    let progression = modal_transformer.apply_sequence(
        start_chord,
        &input.transforms[0..input.length.min(input.transforms.len())],
    );

    println!("\nModal progression ({} transformations):", progression.len() - 1);
    for (i, chord) in progression.iter().enumerate() {
        if i == 0 {
            println!("  Start: {}", chord);
        } else {
            println!(
                "  {}: {} (via {})",
                i,
                chord,
                match &input.transforms[(i - 1) % input.transforms.len()] {
                    Transform::P => "P",
                    Transform::R => "R",
                    Transform::L => "L",
                    Transform::Compound(_) => "compound",
                }
            );
        }
    }

    // Generate MIDI
    let mut composition = Composition::new(110);
    composition.add_harmony_track(&progression, 4, 2);

    composition.save(output)?;
    println!("\nSaved modal exploration to {}", output);
    Ok(())
}
