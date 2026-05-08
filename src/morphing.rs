//! Pattern morphing and interpolation
//!
//! This module provides tools for smoothly transitioning between different
//! musical patterns, creating seamless morphing effects and style
//! interpolation.

use std::fmt::Display;

use crate::rhythm::{Beat, DrumEvent, RhythmPattern};
use crate::{Chord, Transform};

/// Morphs between two rhythm patterns over time
pub struct MorphingRhythmPattern {
    pattern_a: Box<dyn RhythmPattern>,
    pattern_b: Box<dyn RhythmPattern>,
    morph_progress: f64, // 0.0 = fully A, 1.0 = fully B
    morph_duration_bars: usize,
    current_bar: usize,
    auto_morph: bool,
}

impl MorphingRhythmPattern {
    /// Create a new morphing rhythm pattern
    pub fn new(
        pattern_a: Box<dyn RhythmPattern>,
        pattern_b: Box<dyn RhythmPattern>,
        morph_duration_bars: usize,
    ) -> Self {
        Self {
            pattern_a,
            pattern_b,
            morph_progress: 0.0,
            morph_duration_bars,
            current_bar: 0,
            auto_morph: true,
        }
    }

    /// Disable automatic morphing over time
    pub fn with_manual_morph(mut self) -> Self {
        self.auto_morph = false;
        self
    }

    /// Manually set the morph progress (0.0 to 1.0)
    pub fn set_morph_progress(&mut self, progress: f64) {
        self.morph_progress = progress.clamp(0.0, 1.0);
    }

    /// Interpolate between two sets of drum events
    fn interpolate_events_with_progress(
        &self,
        events_a: Vec<DrumEvent>,
        events_b: Vec<DrumEvent>,
        morph_progress: f64,
    ) -> Vec<DrumEvent> {
        let mut result = Vec::new();

        // Probability-based blending
        let use_a_probability = 1.0 - morph_progress;

        // Add events from pattern A with decreasing probability
        for mut event in events_a {
            if fastrand::f64() < use_a_probability {
                // Fade velocity as we morph away
                event.velocity = (event.velocity as f64 * use_a_probability) as u8;
                result.push(event);
            }
        }

        // Add events from pattern B with increasing probability
        for mut event in events_b {
            if fastrand::f64() < morph_progress {
                // Fade velocity as we morph in
                event.velocity = (event.velocity as f64 * morph_progress) as u8;
                result.push(event);
            }
        }

        // Interpolate timing for overlapping events
        self.blend_overlapping_events(result)
    }

    /// Blend events that occur at similar times
    fn blend_overlapping_events(&self, mut events: Vec<DrumEvent>) -> Vec<DrumEvent> {
        // Sort by beat position
        events.sort_by(|a, b| a.beat.0.partial_cmp(&b.beat.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut blended = Vec::new();
        let mut i = 0;

        while i < events.len() {
            let current = events[i];
            let mut similar_events = vec![current];

            // Find events with the same voice and similar timing
            let mut j = i + 1;
            while j < events.len() {
                let other = events[j];
                if other.voice == current.voice && (other.beat.0 - current.beat.0).abs() < 0.125 {
                    similar_events.push(other);
                    j += 1;
                } else {
                    break;
                }
            }

            if similar_events.len() > 1 {
                // Blend multiple similar events
                let avg_beat = similar_events.iter().map(|e| e.beat.0).sum::<f64>() / similar_events.len() as f64;
                let max_velocity = similar_events.iter().map(|e| e.velocity).max().unwrap_or(64);

                blended.push(DrumEvent {
                    voice: current.voice,
                    beat: Beat(avg_beat),
                    velocity: max_velocity,
                });
            } else {
                blended.push(current);
            }

            i = j;
        }

        blended
    }
}

impl RhythmPattern for MorphingRhythmPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        // Calculate morph progress for this bar
        let morph_progress = if self.auto_morph {
            let cycle_position = (bar % (self.morph_duration_bars * 2)) as f64;

            if cycle_position < self.morph_duration_bars as f64 {
                // Morphing from A to B
                cycle_position / self.morph_duration_bars as f64
            } else {
                // Morphing from B back to A
                2.0 - (cycle_position / self.morph_duration_bars as f64)
            }
        } else {
            self.morph_progress
        };

        // Get events from both patterns
        let events_a = self.pattern_a.events_for_bar(bar);
        let events_b = self.pattern_b.events_for_bar(bar);

        // Interpolate between them
        self.interpolate_events_with_progress(events_a, events_b, morph_progress)
    }

    fn pattern_length(&self) -> usize {
        // Use the larger of the two pattern lengths
        self.pattern_a.pattern_length().max(self.pattern_b.pattern_length())
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(MorphingRhythmPattern {
            pattern_a: self.pattern_a.clone_box(),
            pattern_b: self.pattern_b.clone_box(),
            morph_progress: self.morph_progress,
            morph_duration_bars: self.morph_duration_bars,
            current_bar: self.current_bar,
            auto_morph: self.auto_morph,
        })
    }
}

impl Display for MorphingRhythmPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "morphing(A -> B, {:.2})", self.morph_progress)
    }
}

/// Morphs between different harmonic progression patterns
#[derive(Debug, Clone)]
pub struct MorphingHarmonyPattern {
    transforms_a: Vec<Transform>,
    transforms_b: Vec<Transform>,
    morph_progress: f64,
    interpolation_method: HarmonyInterpolationMethod,
}

#[derive(Debug, Clone, Copy)]
pub enum HarmonyInterpolationMethod {
    /// Switch between patterns with probability
    Probabilistic,
    /// Blend transform sequences
    Sequential,
    /// Choose transforms based on harmonic similarity
    HarmonicallySmooth,
}

impl MorphingHarmonyPattern {
    /// Create a new morphing harmony pattern
    pub fn new(transforms_a: Vec<Transform>, transforms_b: Vec<Transform>, method: HarmonyInterpolationMethod) -> Self {
        Self {
            transforms_a,
            transforms_b,
            morph_progress: 0.0,
            interpolation_method: method,
        }
    }

    /// Set the morph progress
    pub fn set_morph_progress(&mut self, progress: f64) {
        self.morph_progress = progress.clamp(0.0, 1.0);
    }

    /// Get the next transform in the morphed sequence
    pub fn get_transform(&self, position: usize, current_chord: Chord) -> Option<Transform> {
        match self.interpolation_method {
            HarmonyInterpolationMethod::Probabilistic => self.probabilistic_blend(position),
            HarmonyInterpolationMethod::Sequential => self.sequential_blend(position),
            HarmonyInterpolationMethod::HarmonicallySmooth => self.harmonically_smooth_blend(position, current_chord),
        }
    }

    /// Probabilistically choose between pattern A and B transforms
    fn probabilistic_blend(&self, position: usize) -> Option<Transform> {
        let use_b = fastrand::f64() < self.morph_progress;

        let transforms = if use_b { &self.transforms_b } else { &self.transforms_a };

        if transforms.is_empty() {
            None
        } else {
            Some(transforms[position % transforms.len()].clone())
        }
    }

    /// Sequentially blend patterns by interpolating their positions
    fn sequential_blend(&self, position: usize) -> Option<Transform> {
        if self.transforms_a.is_empty() && self.transforms_b.is_empty() {
            return None;
        }

        // Interpolate position in both sequences
        let pos_a = if self.transforms_a.is_empty() {
            0
        } else {
            position % self.transforms_a.len()
        };
        let pos_b = if self.transforms_b.is_empty() {
            0
        } else {
            position % self.transforms_b.len()
        };

        // Weighted selection based on morph progress
        if fastrand::f64() < (1.0 - self.morph_progress) && !self.transforms_a.is_empty() {
            Some(self.transforms_a[pos_a].clone())
        } else if !self.transforms_b.is_empty() {
            Some(self.transforms_b[pos_b].clone())
        } else if !self.transforms_a.is_empty() {
            Some(self.transforms_a[pos_a].clone())
        } else {
            None
        }
    }

    /// Choose transforms that create smooth harmonic motion
    fn harmonically_smooth_blend(&self, position: usize, _current_chord: Chord) -> Option<Transform> {
        // For now, use sequential blend
        // In future, could analyze which transform creates smoother voice leading
        self.sequential_blend(position)
    }
}

/// Smooth interpolation utility functions
pub struct PatternInterpolator;

impl PatternInterpolator {
    /// Linear interpolation between two f64 values
    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a * (1.0 - t) + b * t
    }

    /// Smooth step interpolation (ease in/out)
    pub fn smoothstep(t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// Smooth velocity interpolation between drum events
    pub fn interpolate_velocity(vel_a: u8, vel_b: u8, t: f64) -> u8 {
        let smooth_t = Self::smoothstep(t);
        (Self::lerp(vel_a as f64, vel_b as f64, smooth_t) as u8).clamp(0, 127)
    }

    /// Interpolate beat position
    pub fn interpolate_beat(beat_a: f64, beat_b: f64, t: f64) -> f64 {
        Self::lerp(beat_a, beat_b, Self::smoothstep(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rhythm::{DrumVoice, EuclideanPattern};

    #[test]
    fn morphing_rhythm_pattern_creation() {
        let pattern_a = Box::new(EuclideanPattern::new(DrumVoice::Kick, 8, 16));
        let pattern_b = Box::new(EuclideanPattern::new(DrumVoice::Kick, 6, 16));

        let morphing = MorphingRhythmPattern::new(pattern_a, pattern_b, 8);
        assert_eq!(morphing.morph_duration_bars, 8);
        assert_eq!(morphing.morph_progress, 0.0);
    }

    #[test]
    fn morphing_harmony_pattern_creation() {
        let transforms_a = vec![Transform::P, Transform::L];
        let transforms_b = vec![Transform::R, Transform::P];

        let morphing = MorphingHarmonyPattern::new(
            transforms_a.clone(),
            transforms_b.clone(),
            HarmonyInterpolationMethod::Probabilistic,
        );

        assert_eq!(morphing.transforms_a, transforms_a);
        assert_eq!(morphing.transforms_b, transforms_b);
    }

    #[test]
    fn pattern_interpolator_lerp() {
        assert_eq!(PatternInterpolator::lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(PatternInterpolator::lerp(10.0, 20.0, 0.0), 10.0);
        assert_eq!(PatternInterpolator::lerp(10.0, 20.0, 1.0), 20.0);
    }

    #[test]
    fn pattern_interpolator_smoothstep() {
        let smooth_0 = PatternInterpolator::smoothstep(0.0);
        let smooth_1 = PatternInterpolator::smoothstep(1.0);
        let smooth_half = PatternInterpolator::smoothstep(0.5);

        assert_eq!(smooth_0, 0.0);
        assert_eq!(smooth_1, 1.0);
        assert_eq!(smooth_half, 0.5); // At 0.5, smoothstep equals linear
        assert!((0.0..=1.0).contains(&smooth_half));
    }

    #[test]
    fn velocity_interpolation() {
        let vel_a = 60;
        let vel_b = 100;

        let interpolated = PatternInterpolator::interpolate_velocity(vel_a, vel_b, 0.5);
        assert!(interpolated > vel_a && interpolated < vel_b);

        assert_eq!(PatternInterpolator::interpolate_velocity(vel_a, vel_b, 0.0), vel_a);
        assert_eq!(PatternInterpolator::interpolate_velocity(vel_a, vel_b, 1.0), vel_b);
    }

    #[test]
    fn morphing_harmony_probabilistic_blend() {
        let transforms_a = vec![Transform::P];
        let transforms_b = vec![Transform::L];

        let mut morphing =
            MorphingHarmonyPattern::new(transforms_a, transforms_b, HarmonyInterpolationMethod::Probabilistic);

        morphing.set_morph_progress(0.0);
        let transform_at_0 = morphing.get_transform(0, Chord::c_major());
        assert!(transform_at_0.is_some());

        morphing.set_morph_progress(1.0);
        let transform_at_1 = morphing.get_transform(0, Chord::c_major());
        assert!(transform_at_1.is_some());
    }
}
