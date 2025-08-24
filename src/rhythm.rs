//! Rhythm generation and pattern systems

/// Represents a beat position in time (can be fractional for subdivisions)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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

/// A rhythmic event (hit)
#[derive(Debug, Clone)]
pub struct DrumEvent {
    pub voice: DrumVoice,
    pub beat: Beat,
    pub velocity: u8,
}

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

    /// Generate Euclidean rhythm pattern using Bjorklund's algorithm
    /// https://cgm.cs.mcgill.ca/~godfried/publications/banff.pdf
    fn generate_pattern(&self) -> Vec<bool> {
        if self.hits == 0 || self.steps == 0 {
            return vec![false; self.steps];
        }

        if self.hits >= self.steps {
            return vec![true; self.steps];
        }

        let mut pattern = Vec::new();
        let mut remainder = self.hits;
        let mut divisor = self.steps - self.hits;

        let mut level = vec![vec![true]; self.hits];
        level.extend(vec![vec![false]; divisor]);

        while remainder > 1 && divisor > 1 {
            let min_count = divisor.min(remainder);
            for i in 0..min_count {
                let to_append = level[remainder + i].clone();
                level[i].extend(to_append);
            }
            level.truncate(remainder.max(divisor));

            let temp = remainder;
            remainder = divisor % remainder;
            divisor = temp;
        }

        for group in level {
            pattern.extend(group);
        }

        // Apply rotation
        if self.rotation > 0 {
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
    velocity_range: (u8, u8),
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

/// Genre-specific pattern templates
pub struct GenrePatterns;

impl GenrePatterns {
    /// 90s ambient techno - very sparse, breathing
    pub fn ambient_90s() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 2, 4).with_velocity(70)))
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatClosed)
                        .add_point(Beat(1.5), 0.4)
                        .add_point(Beat(3.5), 0.3),
                ))
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Snare)
                        .add_point(Beat(1.25), 0.3)
                        .add_point(Beat(3.25), 0.25),
                )),
        )
    }

    /// 2000s microhouse - shuffled, ghost notes
    pub fn microhouse_2000s() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 2, 4).with_velocity(75)))
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::KickSoft)
                        .add_point(Beat(1.75), 0.8)
                        .add_point(Beat(3.75), 0.7),
                ))
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::HiHatClosed, 8, 16)
                        .with_velocity(45)
                        .with_rotation(1), // Slight shuffle
                )),
        )
    }

    /// 2010s Euclidean patterns
    pub fn euclidean_2010s() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::Kick, 4, 16).with_velocity(80),
                ))
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::Snare, 5, 16)
                        .with_rotation(4)
                        .with_velocity(65),
                ))
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::HiHatClosed, 7, 16)
                        .with_rotation(2)
                        .with_velocity(50),
                )),
        )
    }
}

// Add rand dependency for probability patterns
// Note: You'll need to add `rand = "0.8"` to Cargo.toml

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclidean_patterns() {
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 8);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn can_quantize_beat() {
        let beat = Beat(1.73);
        let quantized = beat.quantize(0.5);
        assert_eq!(quantized.0, 1.5);
    }
}
