//! Neo-Riemannian music theory library for generative composition
//!
//! This library provides tools for exploring neo-Riemannian transformations,
//! generating chord progressions, creating rhythmic patterns, and outputting
//! MIDI files.

pub mod core;
pub mod errors;
pub mod euclidean;
pub mod generation;
pub mod midi;
pub mod rhythm;
pub mod theory;

pub use core::{Chord, ChordQuality, Mode, Note, PitchClass};

pub use errors::LobachevskyError;
pub use euclidean::*;
pub use generation::ProgressionBuilder;
pub use rhythm::{Beat, Duration, RhythmPattern};
pub use theory::{ModalNeoRiemannian, NeoRiemannian, Transform};
