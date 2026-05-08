//! Custom error types for this library.

use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum LobachevskyError {
    #[error("Invalid musical pitch: '{input}'")]
    #[diagnostic(
        code(lobachevsky::invalid_pitch),
        help("Valid pitch format: note letter (A-G) optionally followed by # or b (e.g., 'C', 'F#', 'Bb')")
    )]
    InvalidPitch { input: String },

    #[error("Invalid neo-Riemannian transformation: '{input}'")]
    #[diagnostic(
        code(lobachevsky::invalid_transformation),
        help("Valid transformations: P (Parallel), R (Relative), L (Leading-tone), or compounds like PR, PL, LPL")
    )]
    InvalidTransformation { input: String },

    #[error("Invalid musical mode: '{input}'")]
    #[diagnostic(
        code(lobachevsky::invalid_mode),
        help("Valid modes: ionian, dorian, phrygian, lydian, mixolydian, aeolian, locrian")
    )]
    InvalidMode { input: String },

    #[error("Invalid drum voice: '{voice}'")]
    #[diagnostic(
        code(lobachevsky::invalid_drum_voice),
        help(
            "Valid drum voices: kick, kick_soft, snare, rim, hihat_closed, hihat_open, shaker, ride, clap, percussion"
        )
    )]
    InvalidDrumVoice { voice: String },

    #[error("Unknown rhythm style: '{style}'")]
    #[diagnostic(
        code(lobachevsky::unknown_rhythm_style),
        help("Available styles: 90s, 2000s, 2010s, detroit, berlin, minimal, acid, deep"),
        url("https://github.com/ceejbot/lobachevsky#rhythm-styles")
    )]
    UnknownRhythmStyle { style: String },

    #[error("Euclidean algorithm input error: steps: {steps} < pulses: {pulses}")]
    #[diagnostic(
        code(lobachevsky::euclidean_error),
        help("The number of steps must be larger than or equal to the number of pulses.")
    )]
    EuclideanInputs { steps: usize, pulses: usize },

    #[error("Failed to read directory")]
    #[diagnostic(
        code(lobachevsky::directory_read_error),
        help("Check that the directory exists and you have read permissions")
    )]
    DirectoryReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read pattern file")]
    #[diagnostic(
        code(lobachevsky::pattern_file_error),
        help("Check that the file exists and is a valid TOML pattern definition")
    )]
    PatternFileError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid pattern definition")]
    #[diagnostic(
        code(lobachevsky::pattern_parse_error),
        help("Check the TOML syntax and ensure all required fields are present"),
        url("https://github.com/ceejbot/lobachevsky#pattern-format")
    )]
    PatternParseError {
        #[source]
        source: toml::de::Error,
    },

    #[error("File I/O error")]
    #[diagnostic(code(lobachevsky::file_error))]
    FileError(#[from] std::io::Error),

    #[error("TOML parsing error")]
    #[diagnostic(
        code(lobachevsky::toml_error),
        help("Check the TOML syntax - common issues include missing quotes, invalid types, or unclosed brackets")
    )]
    TomlError(#[from] toml::de::Error),

    #[error("Parse error: {message}")]
    #[diagnostic(code(lobachevsky::parse_error), help("Check the input format and try again"))]
    ParseError { message: String },

    #[error("TUI error: {message}")]
    #[diagnostic(code(lobachevsky::tui_error), help("Check terminal capabilities and try again"))]
    TuiError { message: String },

    #[error("File I/O error at {path}")]
    #[diagnostic(code(lobachevsky::file_io_error))]
    FileIo {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
