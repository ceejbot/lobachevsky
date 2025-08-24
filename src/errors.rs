//! Custom error types for this library.

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum LobachevskyError {
    #[error("The string {0} is an invalid representation of a musical pitch.")]
    InvalidPitch(String),

    #[error("The string {0} is an invalid representation of a neo-Reimannian transformation.")]
    InvalidTransformation(String),

    #[error("The string {0} is an invalid representation of a musical mode.")]
    InvalidMode(String),

    #[error("Unknown style {0}. Lobachevsky knows 90s, 2000s, or 2010s")]
    UnknownRhythmStyle(String),

    #[error(transparent)]
    FileError(#[from] std::io::Error),
}
