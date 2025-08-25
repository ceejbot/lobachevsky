//! Genre patterns in code.

use super::*;

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

    /// Detroit techno - raw, driving, boomy kicks
    pub fn detroit_techno() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Strong 4/4 kick
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(90)))
                // Sparse hi-hats with emphasis on off-beats
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatOpen)
                        .add_point(Beat(0.5), 0.8)
                        .add_point(Beat(1.5), 0.7)
                        .add_point(Beat(2.5), 0.8)
                        .add_point(Beat(3.5), 0.6),
                ))
                // Occasional rim shots for texture
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Rim)
                        .add_point(Beat(1.75), 0.3)
                        .add_point(Beat(3.25), 0.4),
                )),
        )
    }

    /// Berlin dub techno - deep, atmospheric, spacious
    pub fn berlin_dub_techno() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Subdued kick pattern
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(75)))
                // Very sparse, delayed snare hits
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Snare)
                        .add_point(Beat(1.0), 0.2)
                        .add_point(Beat(3.0), 0.15),
                ))
                // Minimal closed hats with micro-timing variations
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatClosed)
                        .add_point(Beat(0.25), 0.3)
                        .add_point(Beat(1.25), 0.35)
                        .add_point(Beat(2.25), 0.3)
                        .add_point(Beat(3.75), 0.4),
                ))
                // Occasional shaker for atmosphere
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Shaker)
                        .add_point(Beat(0.75), 0.2)
                        .add_point(Beat(2.75), 0.25),
                )),
        )
    }

    /// Minimal Berlin - stripped-down, hypnotic
    pub fn minimal_berlin() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Basic kick pattern
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(80)))
                // Clap on 2 and 4 with slight variation
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Clap)
                        .add_point(Beat(1.0), 0.9)
                        .add_point(Beat(3.0), 0.85),
                ))
                // Rolling hi-hat pattern using Euclidean distribution
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::HiHatClosed, 11, 16)
                        .with_velocity(45)
                        .with_rotation(3),
                ))
                // Subtle ride for movement
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Ride)
                        .add_point(Beat(0.5), 0.3)
                        .add_point(Beat(2.5), 0.35),
                )),
        )
    }

    /// Acid minimal - 303-influenced with syncopation
    pub fn acid_minimal() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Kick pattern with occasional ghost kicks
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(85)))
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::KickSoft)
                        .add_point(Beat(0.75), 0.6)
                        .add_point(Beat(2.75), 0.5),
                ))
                // Syncopated snare/rim pattern
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Snare)
                        .add_point(Beat(1.25), 0.7)
                        .add_point(Beat(3.25), 0.8),
                ))
                // Complex hi-hat pattern with 3-over-4 feel
                .add_layer(Box::new(
                    EuclideanPattern::new(DrumVoice::HiHatClosed, 5, 8)
                        .with_velocity(50)
                        .with_rotation(1),
                ))
                // Open hat accents
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatOpen)
                        .add_point(Beat(1.5), 0.5)
                        .add_point(Beat(3.0), 0.4),
                )),
        )
    }

    /// Deep minimal - subdued with ghost notes
    pub fn deep_minimal() -> Box<dyn RhythmPattern> {
        Box::new(
            LayeredPattern::new()
                // Soft, deep kick pattern
                .add_layer(Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4).with_velocity(70)))
                // Very soft ghost kicks for swing
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::KickSoft)
                        .add_point(Beat(0.5), 0.3)
                        .add_point(Beat(1.75), 0.4)
                        .add_point(Beat(2.5), 0.3)
                        .add_point(Beat(3.75), 0.35),
                ))
                // Minimal rim pattern
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Rim)
                        .add_point(Beat(1.0), 0.5)
                        .add_point(Beat(3.0), 0.45),
                ))
                // Very sparse, quiet hi-hats
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::HiHatClosed)
                        .add_point(Beat(0.25), 0.3)
                        .add_point(Beat(0.75), 0.25)
                        .add_point(Beat(2.25), 0.3)
                        .add_point(Beat(2.75), 0.25),
                ))
                // Occasional percussion for texture
                .add_layer(Box::new(
                    ProbabilityPattern::new(DrumVoice::Percussion)
                        .add_point(Beat(1.5), 0.2)
                        .add_point(Beat(3.5), 0.15),
                )),
        )
    }
}
