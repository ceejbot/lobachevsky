pub mod algorithmic;
pub mod extended;
pub mod hexatonic;
pub mod in_c;
pub mod modal;
pub mod progression;

use crate::{LobachevskyError, Mode, PitchClass, Transform};

// Helper functions

fn parse_transforms(pattern_str: &str) -> Result<Vec<Transform>, LobachevskyError> {
    pattern_str
        .split(',')
        .map(|s| Transform::try_from(s.trim()))
        .collect::<Result<Vec<_>, _>>()
}

fn parse_mode_and_tonic(
    mode: Option<&str>,
    tonic: Option<&str>,
) -> Result<(Option<Mode>, Option<PitchClass>), LobachevskyError> {
    match (mode, tonic) {
        (Some(mode_str), Some(tonic_str)) => {
            let parsed_mode = Mode::try_from(mode_str)?;
            let parsed_tonic = PitchClass::try_from(tonic_str)?;
            Ok((Some(parsed_mode), Some(parsed_tonic)))
        }
        (None, None) => Ok((None, None)),
        _ => Err(LobachevskyError::ParseError {
            message: "Both mode and tonic must be specified for modal constraints".to_string(),
        }),
    }
}
