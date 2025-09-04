//! Application state, state transitions, and other logic

use std::path::PathBuf;

use crate::LobachevskyError;
use crate::generators::in_c::{InCConfig, InCGenerator};
use crate::melody::InCPatterns;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectedField {
    NumPerformers,
    Duration,
    Tempo,
    IncludePulse,
    Variation,
    CanonProbability,
    TimingFlex,
}

impl SelectedField {
    pub fn next(&self) -> Self {
        match self {
            Self::NumPerformers => Self::Duration,
            Self::Duration => Self::Tempo,
            Self::Tempo => Self::IncludePulse,
            Self::IncludePulse => Self::Variation,
            Self::Variation => Self::CanonProbability,
            Self::CanonProbability => Self::TimingFlex,
            Self::TimingFlex => Self::NumPerformers,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::NumPerformers => Self::TimingFlex,
            Self::Duration => Self::NumPerformers,
            Self::Tempo => Self::Duration,
            Self::IncludePulse => Self::Tempo,
            Self::Variation => Self::IncludePulse,
            Self::CanonProbability => Self::Variation,
            Self::TimingFlex => Self::CanonProbability,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GenerationStatus {
    Idle,
    Generating,
    Success(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct State {
    pub config: InCConfig,
    pub selected_field: SelectedField,
    pub should_exit: bool,
    pub show_help: bool,
    pub generation_status: GenerationStatus,
    pub output_path: PathBuf,
}

impl State {
    pub fn new() -> Result<Self, LobachevskyError> {
        Ok(Self {
            config: InCConfig::default(),
            selected_field: SelectedField::NumPerformers,
            should_exit: false,
            show_help: false,
            generation_status: GenerationStatus::Idle,
            output_path: PathBuf::from("in_c_performance.mid"),
        })
    }

    pub fn reset_to_defaults(&mut self) {
        self.config = InCConfig::default();
        self.generation_status = GenerationStatus::Idle;
    }

    pub fn increment_field(&mut self, large_step: bool) {
        match self.selected_field {
            SelectedField::NumPerformers => {
                let step = if large_step { 5 } else { 1 };
                self.config.num_performers = (self.config.num_performers + step).min(20);
            }
            SelectedField::Duration => {
                let step = if large_step { 5.0 } else { 1.0 };
                self.config.duration_minutes = (self.config.duration_minutes + step).min(60.0);
            }
            SelectedField::Tempo => {
                let step = if large_step { 10 } else { 1 };
                self.config.tempo = (self.config.tempo + step).min(180);
            }
            SelectedField::IncludePulse => {
                self.config.include_pulse = !self.config.include_pulse;
            }
            SelectedField::Variation => {
                let step = if large_step { 0.1 } else { 0.01 };
                self.config.variation = (self.config.variation + step).min(1.0);
            }
            SelectedField::CanonProbability => {
                let step = if large_step { 0.1 } else { 0.01 };
                self.config.canon_probability = (self.config.canon_probability + step).min(1.0);
            }
            SelectedField::TimingFlex => {
                let step = if large_step { 0.1 } else { 0.01 };
                self.config.timing_flex = (self.config.timing_flex + step).min(1.0);
            }
        }
    }

    pub fn decrement_field(&mut self, large_step: bool) {
        match self.selected_field {
            SelectedField::NumPerformers => {
                let step = if large_step { 5 } else { 1 };
                self.config.num_performers = self.config.num_performers.saturating_sub(step).max(1);
            }
            SelectedField::Duration => {
                let step = if large_step { 5.0 } else { 1.0 };
                self.config.duration_minutes = (self.config.duration_minutes - step).max(5.0);
            }
            SelectedField::Tempo => {
                let step = if large_step { 10 } else { 1 };
                self.config.tempo = self.config.tempo.saturating_sub(step).max(60);
            }
            SelectedField::IncludePulse => {
                self.config.include_pulse = !self.config.include_pulse;
            }
            SelectedField::Variation => {
                let step = if large_step { 0.1 } else { 0.01 };
                self.config.variation = (self.config.variation - step).max(0.0);
            }
            SelectedField::CanonProbability => {
                let step = if large_step { 0.1 } else { 0.01 };
                self.config.canon_probability = (self.config.canon_probability - step).max(0.0);
            }
            SelectedField::TimingFlex => {
                let step = if large_step { 0.1 } else { 0.01 };
                self.config.timing_flex = (self.config.timing_flex - step).max(0.0);
            }
        }
    }

    pub fn toggle_boolean_field(&mut self) {
        if self.selected_field == SelectedField::IncludePulse {
            self.config.include_pulse = !self.config.include_pulse;
        }
    }

    pub fn generate(&mut self) -> Result<(), LobachevskyError> {
        self.generation_status = GenerationStatus::Generating;

        // Load the In C patterns from the TOML file
        let patterns_file = std::path::Path::new("in_c_patterns.toml");
        let patterns_content = std::fs::read_to_string(patterns_file).map_err(LobachevskyError::FileError)?;
        let patterns = InCPatterns::from_toml(&patterns_content)?;

        // Create the generator
        let mut generator = InCGenerator::new(self.config.clone(), patterns);

        // Generate the MIDI file
        match generator.generate() {
            Ok(midi_file) => {
                midi_file.save(&self.output_path)?;
                self.generation_status =
                    GenerationStatus::Success(format!("Generated: {}", self.output_path.display()));
                Ok(())
            }
            Err(e) => {
                self.generation_status = GenerationStatus::Error(e.to_string());
                Err(e)
            }
        }
    }
}
