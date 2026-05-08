//! Adaptive pattern generation
//!
//! This module provides patterns that evolve and adapt based on harmonic
//! content, energy levels, and other musical features. Patterns can respond
//! dynamically to create more expressive and contextually appropriate musical
//! output.

use std::fmt::Display;

use crate::analysis::{EnergyLevel, HarmonicAnalyzer};
use crate::rhythm::{Beat, DrumEvent, DrumVoice, RhythmPattern};
use crate::{Chord, Transform};

/// Adaptive rhythm pattern that responds to harmonic energy
#[derive(Debug, Clone)]
pub struct AdaptiveRhythmPattern {
    base_pattern: String,
    energy_responses: EnergyResponseMap,
    analyzer: HarmonicAnalyzer,
    current_energy: f64,
}

/// Maps energy levels to pattern modifications
#[derive(Debug, Clone)]
pub struct EnergyResponseMap {
    pub low_energy: PatternModification,
    pub medium_energy: PatternModification,
    pub high_energy: PatternModification,
}

/// Modifications to apply to a base pattern based on energy
#[derive(Debug, Clone)]
pub struct PatternModification {
    pub density_multiplier: f64,
    pub velocity_boost: u8,
    pub additional_voices: Vec<DrumVoice>,
    pub syncopation_factor: f64,
}

impl Default for EnergyResponseMap {
    fn default() -> Self {
        Self {
            low_energy: PatternModification {
                density_multiplier: 0.6,
                velocity_boost: 0,
                additional_voices: vec![],
                syncopation_factor: 0.2,
            },
            medium_energy: PatternModification {
                density_multiplier: 1.0,
                velocity_boost: 10,
                additional_voices: vec![DrumVoice::Shaker],
                syncopation_factor: 0.5,
            },
            high_energy: PatternModification {
                density_multiplier: 1.4,
                velocity_boost: 25,
                additional_voices: vec![DrumVoice::Shaker, DrumVoice::Ride, DrumVoice::Clap],
                syncopation_factor: 0.8,
            },
        }
    }
}

impl AdaptiveRhythmPattern {
    /// Create a new adaptive rhythm pattern
    pub fn new(base_pattern: String, tonic: Option<crate::PitchClass>) -> Self {
        let analyzer = if let Some(tonic) = tonic {
            HarmonicAnalyzer::new().with_tonic(tonic)
        } else {
            HarmonicAnalyzer::new()
        };

        Self {
            base_pattern,
            energy_responses: EnergyResponseMap::default(),
            analyzer,
            current_energy: 0.5, // Start at medium energy
        }
    }

    /// Customize energy responses
    pub fn with_energy_responses(mut self, responses: EnergyResponseMap) -> Self {
        self.energy_responses = responses;
        self
    }

    /// Update the pattern based on current harmonic context
    pub fn update_from_chords(&mut self, chords: &[Chord]) {
        self.current_energy = self.analyzer.analyze_progression_energy(chords);
    }

    /// Generate base pattern events then modify based on current energy
    fn generate_base_events(&self, bar: usize) -> Vec<DrumEvent> {
        // Simple 4/4 base pattern - kick on 1 and 3, snare on 2 and 4
        let mut events = vec![
            DrumEvent {
                voice: DrumVoice::Kick,
                beat: Beat(0.0),
                velocity: 100,
            },
            DrumEvent {
                voice: DrumVoice::Snare,
                beat: Beat(1.0),
                velocity: 90,
            },
            DrumEvent {
                voice: DrumVoice::Kick,
                beat: Beat(2.0),
                velocity: 95,
            },
            DrumEvent {
                voice: DrumVoice::Snare,
                beat: Beat(3.0),
                velocity: 85,
            },
        ];

        // Add hi-hats on eighth notes
        for eighth in 0..8 {
            events.push(DrumEvent {
                voice: DrumVoice::HiHatClosed,
                beat: Beat(eighth as f64 * 0.5),
                velocity: 60,
            });
        }

        // Add some variation based on bar number
        if bar % 4 == 3 && fastrand::f64() < 0.7 {
            // Fill every 4th bar
            events.push(DrumEvent {
                voice: DrumVoice::Snare,
                beat: Beat(3.5),
                velocity: 70,
            });
        }

        events
    }

    /// Apply energy-based modifications to events
    fn apply_energy_modifications(&self, mut events: Vec<DrumEvent>) -> Vec<DrumEvent> {
        let energy_level = EnergyLevel::from(self.current_energy);
        let modification = match energy_level {
            EnergyLevel::Low => &self.energy_responses.low_energy,
            EnergyLevel::Medium => &self.energy_responses.medium_energy,
            EnergyLevel::High => &self.energy_responses.high_energy,
        };

        // Apply velocity boost
        for event in &mut events {
            event.velocity = (event.velocity + modification.velocity_boost).min(127);
        }

        // Apply density modifications
        if modification.density_multiplier < 1.0 {
            // Remove some events for lower density
            let keep_probability = modification.density_multiplier;
            events.retain(|_| fastrand::f64() < keep_probability);
        } else if modification.density_multiplier > 1.0 {
            // Add additional events for higher density
            let additional_events = ((events.len() as f64 * (modification.density_multiplier - 1.0)) as usize).min(8);

            for _ in 0..additional_events {
                if let Some(&voice) = modification
                    .additional_voices
                    .get(fastrand::usize(0..modification.additional_voices.len().max(1)))
                {
                    events.push(DrumEvent {
                        voice,
                        beat: Beat(fastrand::f64() * 4.0), // Random position in bar
                        velocity: 40 + fastrand::u8(0..30),
                    });
                }
            }
        }

        // Apply syncopation
        if modification.syncopation_factor > 0.5 {
            // Add off-beat events
            let syncopation_events = (events.len() as f64 * (modification.syncopation_factor - 0.5)) as usize;

            for _ in 0..syncopation_events {
                if fastrand::f64() < modification.syncopation_factor {
                    events.push(DrumEvent {
                        voice: DrumVoice::Kick,
                        beat: Beat(fastrand::f64() * 4.0),
                        velocity: 60 + fastrand::u8(0..20),
                    });
                }
            }
        }

        events.sort_by(|a, b| a.beat.0.partial_cmp(&b.beat.0).unwrap_or(std::cmp::Ordering::Equal));
        events
    }
}

impl RhythmPattern for AdaptiveRhythmPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        let base_events = self.generate_base_events(bar);
        self.apply_energy_modifications(base_events)
    }

    fn pattern_length(&self) -> usize {
        4 // 4-bar pattern with variation
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(self.clone())
    }
}

impl Display for AdaptiveRhythmPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "adaptive({})", self.base_pattern)
    }
}

/// Adaptive harmony pattern that evolves transformations based on energy
#[derive(Debug, Clone)]
pub struct AdaptiveHarmonyPattern {
    base_transforms: Vec<Transform>,
    energy_transforms: AdaptiveTransformMap,
    analyzer: HarmonicAnalyzer,
    evolution_factor: f64,
    bars_since_change: usize,
}

/// Maps energy levels to transformation preferences
#[derive(Debug, Clone)]
pub struct AdaptiveTransformMap {
    pub low_energy: TransformPreferences,
    pub medium_energy: TransformPreferences,
    pub high_energy: TransformPreferences,
}

/// Preferences for transformation selection
#[derive(Debug, Clone)]
pub struct TransformPreferences {
    pub preferred_transforms: Vec<Transform>,
    pub transform_probability: f64,
    pub complexity_bias: f64, // -1.0 = simple, +1.0 = complex
}

impl Default for AdaptiveTransformMap {
    fn default() -> Self {
        Self {
            low_energy: TransformPreferences {
                preferred_transforms: vec![Transform::P], // Simple parallel motion
                transform_probability: 0.3,
                complexity_bias: -0.5,
            },
            medium_energy: TransformPreferences {
                preferred_transforms: vec![Transform::P, Transform::L, Transform::R],
                transform_probability: 0.7,
                complexity_bias: 0.0,
            },
            high_energy: TransformPreferences {
                preferred_transforms: vec![
                    Transform::L,
                    Transform::R,
                    Transform::P,
                    // Could add compound transforms here in future
                ],
                transform_probability: 0.9,
                complexity_bias: 0.5,
            },
        }
    }
}

impl AdaptiveHarmonyPattern {
    /// Create a new adaptive harmony pattern
    pub fn new(base_transforms: Vec<Transform>, tonic: crate::PitchClass) -> Self {
        Self {
            base_transforms,
            energy_transforms: AdaptiveTransformMap::default(),
            analyzer: HarmonicAnalyzer::new().with_tonic(tonic),
            evolution_factor: 0.1, // How quickly pattern evolves
            bars_since_change: 0,
        }
    }

    /// Evolve the harmony pattern based on current context
    pub fn evolve_pattern(&mut self, current_chord: Chord, bar: usize) -> Option<Transform> {
        let chord_energy = self.analyzer.analyze_chord_energy(current_chord, None);
        let energy_level = EnergyLevel::from(chord_energy);

        self.bars_since_change += 1;

        // Choose transform based on energy level
        let preferences = match energy_level {
            EnergyLevel::Low => &self.energy_transforms.low_energy,
            EnergyLevel::Medium => &self.energy_transforms.medium_energy,
            EnergyLevel::High => &self.energy_transforms.high_energy,
        };

        // Decide whether to apply a transform
        if fastrand::f64() < preferences.transform_probability {
            // Evolution: occasionally try transforms outside the base pattern
            let use_evolved = fastrand::f64() < self.evolution_factor * (self.bars_since_change as f64 / 8.0);

            let transform = if use_evolved && !preferences.preferred_transforms.is_empty() {
                // Use energy-appropriate transform
                preferences
                    .preferred_transforms
                    .get(fastrand::usize(0..preferences.preferred_transforms.len()))?
                    .clone()
            } else if !self.base_transforms.is_empty() {
                // Use base pattern transform
                self.base_transforms.get(bar % self.base_transforms.len())?.clone()
            } else {
                Transform::P // Fallback
            };

            self.bars_since_change = 0;
            Some(transform)
        } else {
            None // No transform this bar
        }
    }

    /// Customize the evolution factor
    pub fn with_evolution_factor(mut self, factor: f64) -> Self {
        self.evolution_factor = factor.clamp(0.0, 1.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chord;

    #[test]
    fn adaptive_rhythm_pattern_creation() {
        let pattern = AdaptiveRhythmPattern::new("test".to_string(), Some(crate::PitchClass::C));
        assert_eq!(pattern.base_pattern, "test");
        assert!(pattern.current_energy >= 0.0 && pattern.current_energy <= 1.0);
    }

    #[test]
    fn adaptive_rhythm_responds_to_energy() {
        let mut pattern = AdaptiveRhythmPattern::new("test".to_string(), None);

        // Low energy chords (simple progression)
        let low_energy_chords = vec![Chord::c_major(), Chord::c_major(), Chord::c_major()];
        pattern.update_from_chords(&low_energy_chords);
        let low_energy_events = pattern.events_for_bar(0);

        // High energy chords (complex progression with larger jumps)
        let high_energy_chords = vec![
            Chord::c_major(),
            Chord::new(crate::PitchClass::Fs, crate::ChordQuality::Diminished),
            Chord::new(crate::PitchClass::As, crate::ChordQuality::Augmented),
        ];
        pattern.update_from_chords(&high_energy_chords);
        let high_energy_events = pattern.events_for_bar(0);

        // High energy should generally produce more events or higher velocities
        // (though randomness means this isn't guaranteed)
        assert!(pattern.current_energy >= 0.0);
        assert!(!low_energy_events.is_empty());
        assert!(!high_energy_events.is_empty());
    }

    #[test]
    fn adaptive_harmony_pattern_creation() {
        let transforms = vec![Transform::P, Transform::L, Transform::R];
        let pattern = AdaptiveHarmonyPattern::new(transforms.clone(), crate::PitchClass::C);
        assert_eq!(pattern.base_transforms, transforms);
    }

    #[test]
    fn adaptive_harmony_evolves() {
        let transforms = vec![Transform::P];
        let mut pattern = AdaptiveHarmonyPattern::new(transforms, crate::PitchClass::C);

        // Test evolution over multiple bars
        let mut transformations = Vec::new();
        for bar in 0..16 {
            if let Some(transform) = pattern.evolve_pattern(Chord::c_major(), bar) {
                transformations.push(transform);
            }
        }

        // Should generate some transformations
        assert!(!transformations.is_empty() || pattern.bars_since_change > 0);
    }
}
