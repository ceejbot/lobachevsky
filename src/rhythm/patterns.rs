//! All the types of pattern generators.

use super::*;

/// Trait for rhythm pattern generators
pub trait RhythmPattern: Send + Sync {
    /// Generate events for a specific bar
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent>;

    /// Get the length of the pattern in bars
    fn pattern_length(&self) -> usize {
        1
    }

    /// Clone the pattern (for dynamic dispatch)
    fn clone_box(&self) -> Box<dyn RhythmPattern>;
}

/// Euclidean rhythm generator - distributes hits evenly across steps
#[derive(Debug, Clone)]
pub struct EuclideanPattern {
    voice: DrumVoice,
    hits: usize,
    steps: usize,
    rotation: usize,
    velocity: u8,
}

impl Default for EuclideanPattern {
    fn default() -> Self {
        Self {
            voice: DrumVoice::Kick,
            hits: 0,
            steps: 4,
            rotation: 0,
            velocity: 64,
        }
    }
}

impl EuclideanPattern {
    pub fn new(voice: DrumVoice, hits: usize, steps: usize) -> Self {
        EuclideanPattern {
            voice,
            hits,
            steps,
            rotation: 0,
            velocity: 64,
        }
    }

    pub fn with_rotation(mut self, rotation: usize) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_velocity(mut self, velocity: u8) -> Self {
        self.velocity = velocity;
        self
    }

    /// Generate Euclidean rhythm pattern using the Bresenham algorithm
    /// This distributes hits as evenly as possible across the steps
    fn generate_pattern(&self) -> Vec<bool> {
        use crate::euclidean::{Breshenham, EuclideanRhythm};

        // Handle edge cases that the old implementation allowed
        if self.steps == 0 {
            return Vec::new();
        }

        // Clamp hits to steps (maintain backward compatibility)
        let hits = self.hits.min(self.steps);

        // Use the tested and optimized Bresenham implementation
        let mut pattern = Breshenham::generate(self.steps, hits).unwrap_or_else(|_| vec![false; self.steps]);

        // Apply additional rotation on top of the default "start with beat" rotation
        if self.rotation > 0 && !pattern.is_empty() {
            let rot = self.rotation % pattern.len();
            pattern.rotate_left(rot);
        }

        pattern
    }
}

impl RhythmPattern for EuclideanPattern {
    fn events_for_bar(&self, _bar: usize) -> Vec<DrumEvent> {
        let pattern = self.generate_pattern();
        let mut events = Vec::new();

        for (i, &hit) in pattern.iter().enumerate() {
            if hit {
                let beat = Beat(i as f64 * 4.0 / self.steps as f64);
                events.push(DrumEvent {
                    voice: self.voice,
                    beat,
                    velocity: self.velocity,
                });
            }
        }

        events
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(self.clone())
    }
}

/// Probability-based pattern generator
#[derive(Debug, Clone)]
pub struct ProbabilityPattern {
    voice: DrumVoice,
    densities: Vec<(Beat, f64)>, // (beat_position, probability)
    pub velocity_range: (u8, u8),
}

impl ProbabilityPattern {
    pub fn new(voice: DrumVoice) -> Self {
        ProbabilityPattern {
            voice,
            densities: Vec::new(),
            velocity_range: (40, 80),
        }
    }

    pub fn add_point(mut self, beat: Beat, probability: f64) -> Self {
        self.densities.push((beat, probability.clamp(0.0, 1.0)));
        self
    }

    pub fn sparse(voice: DrumVoice) -> Self {
        Self::new(voice).add_point(Beat(0.0), 0.7).add_point(Beat(2.0), 0.5)
    }

    pub fn dense(voice: DrumVoice) -> Self {
        Self::new(voice)
            .add_point(Beat(0.0), 0.9)
            .add_point(Beat(0.5), 0.6)
            .add_point(Beat(1.0), 0.8)
            .add_point(Beat(1.5), 0.5)
            .add_point(Beat(2.0), 0.9)
            .add_point(Beat(2.5), 0.6)
            .add_point(Beat(3.0), 0.7)
            .add_point(Beat(3.5), 0.4)
    }
}

impl RhythmPattern for ProbabilityPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        use rand::Rng;
        let mut rng = rand::rng();
        let mut events = Vec::new();

        // Add some variation per bar
        let bar_variation = (bar % 4) as f64 * 0.05;

        for (beat, base_prob) in &self.densities {
            let probability = (base_prob + bar_variation).clamp(0.0, 1.0);

            if rng.random::<f64>() < probability {
                let velocity = rng.random_range(self.velocity_range.0..=self.velocity_range.1);
                events.push(DrumEvent {
                    voice: self.voice,
                    beat: *beat,
                    velocity,
                });
            }
        }

        events
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(self.clone())
    }
}

/// Layered pattern combining multiple sub-patterns
pub struct LayeredPattern {
    layers: Vec<Box<dyn RhythmPattern>>,
}

impl Default for LayeredPattern {
    fn default() -> Self {
        Self::new()
    }
}

impl LayeredPattern {
    pub fn new() -> Self {
        LayeredPattern { layers: Vec::new() }
    }

    pub fn add_layer(mut self, pattern: Box<dyn RhythmPattern>) -> Self {
        self.layers.push(pattern);
        self
    }
}

impl Clone for LayeredPattern {
    fn clone(&self) -> Self {
        LayeredPattern {
            layers: self.layers.iter().map(|l| l.clone_box()).collect(),
        }
    }
}

impl RhythmPattern for LayeredPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        let mut all_events = Vec::new();

        for layer in &self.layers {
            all_events.extend(layer.events_for_bar(bar));
        }

        // Sort by beat position
        all_events.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
        all_events
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(self.clone())
    }
}

/// Swing pattern - applies swing timing to an underlying pattern
pub struct SwingPattern {
    base_pattern: Box<dyn RhythmPattern>,
    swing_ratio: f64,
    subdivision: f64,
    swing_accent: Option<u8>,
}

impl SwingPattern {
    pub fn new(base_pattern: Box<dyn RhythmPattern>, swing_ratio: f64, subdivision: f64) -> Self {
        SwingPattern {
            base_pattern,
            swing_ratio,
            subdivision,
            swing_accent: None,
        }
    }

    pub fn with_swing_accent(mut self, accent: u8) -> Self {
        self.swing_accent = Some(accent);
        self
    }

    /// Apply swing timing to a beat position
    fn apply_swing(&self, beat: Beat) -> Beat {
        let beat_pos = beat.0;
        let subdivision_pos = beat_pos % self.subdivision;

        // Only swing off-beats (beats that fall on subdivision boundaries)
        if (subdivision_pos - self.subdivision / 2.0).abs() < 0.001 {
            // This is an off-beat, apply swing
            let swing_adjustment = (self.swing_ratio - 0.5) * self.subdivision;
            Beat(beat_pos + swing_adjustment)
        } else {
            beat
        }
    }
}

impl RhythmPattern for SwingPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        let base_events = self.base_pattern.events_for_bar(bar);

        base_events
            .into_iter()
            .map(|mut event| {
                event.beat = self.apply_swing(event.beat);

                // Apply swing accent if configured
                if let Some(accent) = self.swing_accent {
                    let beat_pos = event.beat.0 % self.subdivision;
                    if (beat_pos - self.subdivision / 2.0).abs() < 0.001 {
                        event.velocity = (event.velocity as u16 + accent as u16).min(127) as u8;
                    }
                }

                event
            })
            .collect()
    }

    fn pattern_length(&self) -> usize {
        self.base_pattern.pattern_length()
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(SwingPattern {
            base_pattern: self.base_pattern.clone_box(),
            swing_ratio: self.swing_ratio,
            subdivision: self.subdivision,
            swing_accent: self.swing_accent,
        })
    }
}

impl std::fmt::Debug for SwingPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwingPattern")
            .field("swing_ratio", &self.swing_ratio)
            .field("subdivision", &self.subdivision)
            .field("swing_accent", &self.swing_accent)
            .finish_non_exhaustive()
    }
}

impl Clone for SwingPattern {
    fn clone(&self) -> Self {
        SwingPattern {
            base_pattern: self.base_pattern.clone_box(),
            swing_ratio: self.swing_ratio,
            subdivision: self.subdivision,
            swing_accent: self.swing_accent,
        }
    }
}

/// Polyrhythmic pattern - applies different time signatures to patterns
pub struct PolyrhythmicPattern {
    base_pattern: Box<dyn RhythmPattern>,
    time_signature: (u16, u16),
    pattern_length: u16,
    bars_per_cycle: f64,
}

impl PolyrhythmicPattern {
    pub fn new(base_pattern: Box<dyn RhythmPattern>, time_signature: (u16, u16), pattern_length: u16) -> Self {
        // Calculate how many 4/4 bars this polyrhythmic pattern spans
        let bars_per_cycle = (time_signature.0 as f64 * pattern_length as f64) / (4.0 * time_signature.1 as f64 / 4.0);

        PolyrhythmicPattern {
            base_pattern,
            time_signature,
            pattern_length,
            bars_per_cycle,
        }
    }

    /// Transform beat timing from base time signature to 4/4
    fn transform_timing(&self, beat: Beat, bar: usize) -> Beat {
        let (num, den) = self.time_signature;
        let polyrhythm_beats_per_bar = num as f64 * (4.0 / den as f64);
        let pattern_position = (bar as f64 * 4.0 + beat.0) % (self.bars_per_cycle * 4.0);

        // Scale timing to polyrhythmic time signature
        Beat(pattern_position * polyrhythm_beats_per_bar / 4.0)
    }
}

impl RhythmPattern for PolyrhythmicPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        let polyrhythm_bar = (bar as f64 / self.bars_per_cycle) as usize % self.pattern_length as usize;
        let base_events = self.base_pattern.events_for_bar(polyrhythm_bar);

        base_events
            .into_iter()
            .map(|mut event| {
                event.beat = self.transform_timing(event.beat, bar);
                event
            })
            .collect()
    }

    fn pattern_length(&self) -> usize {
        self.bars_per_cycle.ceil() as usize
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(PolyrhythmicPattern {
            base_pattern: self.base_pattern.clone_box(),
            time_signature: self.time_signature,
            pattern_length: self.pattern_length,
            bars_per_cycle: self.bars_per_cycle,
        })
    }
}

impl std::fmt::Debug for PolyrhythmicPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolyrhythmicPattern")
            .field("time_signature", &self.time_signature)
            .field("pattern_length", &self.pattern_length)
            .field("bars_per_cycle", &self.bars_per_cycle)
            .finish_non_exhaustive()
    }
}

impl Clone for PolyrhythmicPattern {
    fn clone(&self) -> Self {
        PolyrhythmicPattern {
            base_pattern: self.base_pattern.clone_box(),
            time_signature: self.time_signature,
            pattern_length: self.pattern_length,
            bars_per_cycle: self.bars_per_cycle,
        }
    }
}

/// Groove pattern - applies microtiming and velocity adjustments
pub struct GroovePattern {
    base_pattern: Box<dyn RhythmPattern>,
    groove_type: GrooveType,
    intensity: f64,
    humanization: Option<f64>,
}

impl GroovePattern {
    pub fn new(base_pattern: Box<dyn RhythmPattern>, groove_type: GrooveType, intensity: f64) -> Self {
        GroovePattern {
            base_pattern,
            groove_type,
            intensity,
            humanization: None,
        }
    }

    pub fn with_humanization(mut self, amount: f64) -> Self {
        self.humanization = Some(amount);
        self
    }

    fn apply_groove(&self, event: DrumEvent) -> DrumEvent {
        let mut result = event;

        match &self.groove_type {
            GrooveType::Template { name } => {
                result = self.apply_template_groove(result, name);
            }
            GrooveType::Custom {
                timing_map,
                velocity_map,
            } => {
                result = self.apply_custom_groove(result, timing_map, velocity_map);
            }
        }

        // Apply humanization if specified
        if let Some(humanization) = self.humanization {
            result.beat = result.beat.humanize(humanization * self.intensity);
        }

        result
    }

    fn apply_template_groove(&self, mut event: DrumEvent, template: &str) -> DrumEvent {
        match template {
            "deep_house_shuffle" => {
                // Apply classic deep house shuffle timing
                let beat_mod = event.beat.0 % 1.0;
                if (beat_mod - 0.5).abs() < 0.1 {
                    // Off-beat
                    event.beat = Beat(event.beat.0 + 0.05 * self.intensity);
                    event.velocity = ((event.velocity as f64) * (1.0 - 0.1 * self.intensity)) as u8;
                }
            }
            "dub_delay" => {
                // Add slight delay and velocity reduction for dub feel
                event.beat = Beat(event.beat.0 + 0.02 * self.intensity);
                event.velocity = ((event.velocity as f64) * (1.0 - 0.05 * self.intensity)) as u8;
            }
            _ => {} // Unknown template, no change
        }
        event
    }

    fn apply_custom_groove(
        &self,
        mut event: DrumEvent,
        timing_map: &std::collections::HashMap<String, f64>,
        velocity_map: &Option<std::collections::HashMap<String, f64>>,
    ) -> DrumEvent {
        let beat_key = format!("{:.2}", event.beat.0);

        if let Some(&timing_adjustment) = timing_map.get(&beat_key) {
            event.beat = Beat(event.beat.0 + timing_adjustment * self.intensity);
        }

        if let Some(vel_map) = velocity_map
            && let Some(&velocity_mult) = vel_map.get(&beat_key)
        {
            event.velocity = ((event.velocity as f64) * velocity_mult * self.intensity
                + (event.velocity as f64) * (1.0 - self.intensity)) as u8;
        }

        event
    }
}

impl RhythmPattern for GroovePattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        let base_events = self.base_pattern.events_for_bar(bar);

        base_events.into_iter().map(|event| self.apply_groove(event)).collect()
    }

    fn pattern_length(&self) -> usize {
        self.base_pattern.pattern_length()
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(GroovePattern {
            base_pattern: self.base_pattern.clone_box(),
            groove_type: self.groove_type.clone(),
            intensity: self.intensity,
            humanization: self.humanization,
        })
    }
}

impl std::fmt::Debug for GroovePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroovePattern")
            .field("groove_type", &self.groove_type)
            .field("intensity", &self.intensity)
            .field("humanization", &self.humanization)
            .finish_non_exhaustive()
    }
}

impl Clone for GroovePattern {
    fn clone(&self) -> Self {
        GroovePattern {
            base_pattern: self.base_pattern.clone_box(),
            groove_type: self.groove_type.clone(),
            intensity: self.intensity,
            humanization: self.humanization,
        }
    }
}

/// Isochronic pattern - generates regular pulses at specific Hz for brainwave
/// entrainment
pub struct IsochronicPattern {
    voice: DrumVoice,
    frequency_hz: f64,
    velocity: u8,
    base_pattern: Option<Box<dyn RhythmPattern>>,
    pulse_density: f64, // How many pulses per beat
}

impl IsochronicPattern {
    pub fn new(voice: DrumVoice, frequency_hz: f64) -> Self {
        // Calculate pulse density: frequency_hz pulses per second
        // At 120 BPM, there are 2 beats per second, so pulse_density = frequency_hz / 2
        // For a general tempo: beats_per_second = BPM / 60, so pulse_density =
        // frequency_hz / (BPM/60) We'll use a standard 120 BPM reference:
        // pulse_density = frequency_hz / 2
        let pulse_density = frequency_hz / 2.0;

        IsochronicPattern {
            voice,
            frequency_hz,
            velocity: 30, // Subtle by default
            base_pattern: None,
            pulse_density,
        }
    }

    pub fn with_velocity(mut self, velocity: u8) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_base_pattern(mut self, base_pattern: Box<dyn RhythmPattern>) -> Self {
        self.base_pattern = Some(base_pattern);
        self
    }

    /// Generate isochronic pulses for a bar
    fn generate_isochronic_events(&self, _bar: usize) -> Vec<DrumEvent> {
        let mut events = Vec::new();

        // Generate pulses at the specified frequency
        // Each bar is 4 beats, so we need pulse_density * 4 pulses per bar
        let pulses_per_bar = (self.pulse_density * 4.0).round() as usize;

        if pulses_per_bar == 0 {
            return events;
        }

        let pulse_interval = 4.0 / pulses_per_bar as f64; // Beats between pulses

        for i in 0..pulses_per_bar {
            let beat_position = i as f64 * pulse_interval;
            events.push(DrumEvent {
                voice: self.voice,
                beat: Beat(beat_position),
                velocity: self.velocity,
            });
        }

        events
    }
}

impl RhythmPattern for IsochronicPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent> {
        let mut events = Vec::new();

        // Add base pattern events if present
        if let Some(base) = &self.base_pattern {
            events.extend(base.events_for_bar(bar));
        }

        // Add isochronic pulse events
        events.extend(self.generate_isochronic_events(bar));

        // Sort by beat position
        events.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
        events
    }

    fn pattern_length(&self) -> usize {
        self.base_pattern.as_ref().map_or(1, |p| p.pattern_length())
    }

    fn clone_box(&self) -> Box<dyn RhythmPattern> {
        Box::new(IsochronicPattern {
            voice: self.voice,
            frequency_hz: self.frequency_hz,
            velocity: self.velocity,
            base_pattern: self.base_pattern.as_ref().map(|p| p.clone_box()),
            pulse_density: self.pulse_density,
        })
    }
}

impl std::fmt::Debug for IsochronicPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsochronicPattern")
            .field("voice", &self.voice)
            .field("frequency_hz", &self.frequency_hz)
            .field("velocity", &self.velocity)
            .field("pulse_density", &self.pulse_density)
            .finish_non_exhaustive()
    }
}

impl Clone for IsochronicPattern {
    fn clone(&self) -> Self {
        IsochronicPattern {
            voice: self.voice,
            frequency_hz: self.frequency_hz,
            velocity: self.velocity,
            base_pattern: self.base_pattern.as_ref().map(|p| p.clone_box()),
            pulse_density: self.pulse_density,
        }
    }
}
