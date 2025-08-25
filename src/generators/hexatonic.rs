//! Implementation for the hexatonic generator.

use super::*;
use crate::Chord;
use crate::generation::{HexatonicCycle, HexatonicExplorer};
use crate::midi::Composition;

/// Input for the Hexatonic command
pub struct HexatonicInput {
    pub cycle: HexatonicCycle,
    pub start_chord: Chord,
}

impl HexatonicInput {
    pub fn from_cli(cycle: &str, start: &str) -> Result<Self, LobachevskyError> {
        let start_chord = Chord::try_from(start)?;

        let cycle = match cycle.to_lowercase().as_str() {
            "northern" => HexatonicCycle::Northern,
            "western" => HexatonicCycle::Western,
            "eastern" => HexatonicCycle::Eastern,
            _ => {
                return Err(LobachevskyError::ParseError {
                    message: format!("Invalid cycle type '{}'. Use: northern, western, or eastern", cycle),
                });
            }
        };

        Ok(HexatonicInput { cycle, start_chord })
    }
}

pub fn generate_hexatonic(input: HexatonicInput, output: &str) -> Result<(), LobachevskyError> {
    let explorer = HexatonicExplorer::new();

    if let Some(chords) = explorer.get_cycle(input.cycle, input.start_chord) {
        println!("Hexatonic cycle starting from {}:", input.start_chord);
        for (i, chord) in chords.iter().enumerate() {
            println!("  {}: {}", i + 1, chord);
        }

        // Generate MIDI - repeat the cycle several times
        let mut full_progression = Vec::new();
        for _ in 0..4 {
            full_progression.extend_from_slice(&chords);
        }

        let mut composition = Composition::new(110);
        composition.add_harmony_track(&full_progression, 4, 2);

        composition.save(output)?;
        println!("Saved to {}", output);
        Ok(())
    } else {
        Err(LobachevskyError::ParseError {
            message: "Could not generate hexatonic cycle".to_string(),
        })
    }
}
