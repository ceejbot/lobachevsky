//! The primitive types of rhythm.

use serde::{Deserialize, Serialize};

use crate::LobachevskyError;

/// Represents a beat position in time (can be fractional for subdivisions)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Beat(pub f64);

impl Beat {
    pub fn new(value: f64) -> Self {
        Beat(value)
    }

    /// Quantize to nearest subdivision
    pub fn quantize(&self, subdivision: f64) -> Self {
        Beat((self.0 / subdivision).round() * subdivision)
    }

    /// Add humanization (micro-timing variation)
    pub fn humanize(&self, amount: f64) -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let variation = rng.random_range(-amount..amount);
        Beat(self.0 + variation)
    }
}

/// Represents a duration in beats
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Duration(pub f64);

/// Drum/percussion instrument types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrumVoice {
    Kick,
    KickSoft,
    Snare,
    Rim,
    HiHatClosed,
    HiHatOpen,
    Shaker,
    Ride,
    Clap,
    Percussion,
}

impl DrumVoice {
    /// Get General MIDI note number for this drum
    pub fn midi_note(&self) -> u8 {
        match self {
            DrumVoice::Kick => 36,
            DrumVoice::KickSoft => 35,
            DrumVoice::Snare => 38,
            DrumVoice::Rim => 37,
            DrumVoice::HiHatClosed => 42,
            DrumVoice::HiHatOpen => 46,
            DrumVoice::Shaker => 70,
            DrumVoice::Ride => 51,
            DrumVoice::Clap => 39,
            DrumVoice::Percussion => 69,
        }
    }
}

impl TryFrom<&str> for DrumVoice {
    type Error = LobachevskyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "kick" => Ok(DrumVoice::Kick),
            "kicksoft" | "kick_soft" => Ok(DrumVoice::KickSoft),
            "snare" => Ok(DrumVoice::Snare),
            "rim" => Ok(DrumVoice::Rim),
            "hihatclosed" | "hihat_closed" | "closed_hihat" => Ok(DrumVoice::HiHatClosed),
            "hihatopen" | "hihat_open" | "open_hihat" => Ok(DrumVoice::HiHatOpen),
            "shaker" => Ok(DrumVoice::Shaker),
            "ride" => Ok(DrumVoice::Ride),
            "clap" => Ok(DrumVoice::Clap),
            "percussion" | "perc" => Ok(DrumVoice::Percussion),
            _ => Err(LobachevskyError::InvalidDrumVoice {
                voice: value.to_string(),
            }),
        }
    }
}

/// A rhythmic event (hit)
#[derive(Debug, Clone)]
pub struct DrumEvent {
    pub voice: DrumVoice,
    pub beat: Beat,
    pub velocity: u8,
}
