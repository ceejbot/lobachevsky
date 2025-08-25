//! Rhythm generation and pattern systems

mod beats;
mod genres;
mod library;
mod patterns;

pub use beats::*;
pub use genres::*;
pub use library::*;
pub use patterns::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclidean_patterns_basic() {
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 8);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 3); // Should have 3 hits

        // Check that events are spread evenly
        let positions: Vec<f64> = events.iter().map(|e| e.beat.0).collect();
        println!("3/8 pattern positions: {:?}", positions);
        assert!(positions.len() == 3);
    }

    #[test]
    fn euclidean_edge_cases() {
        // Test zero hits
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 0, 8);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 0);

        // Test zero steps
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 0);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 0);

        // Test hits >= steps
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 8, 8);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 8);

        let pattern = EuclideanPattern::new(DrumVoice::Kick, 10, 8);
        let events = pattern.events_for_bar(0);
        assert_eq!(events.len(), 8);
    }

    #[test]
    fn euclidean_problematic_cases() {
        // These were the cases that caused panics before the fix
        let test_cases = vec![
            (5, 8),   // Original failing case
            (4, 7),   // Another failing case
            (7, 12),  // Complex case
            (3, 5),   // Simple case
            (13, 16), // Many hits
            (1, 16),  // Very sparse
        ];

        for (hits, steps) in test_cases {
            println!("Testing ({}, {})", hits, steps);
            let pattern = EuclideanPattern::new(DrumVoice::Kick, hits, steps);
            let events = pattern.events_for_bar(0);

            // Should not panic and should have correct number of hits
            assert_eq!(
                events.len(),
                hits.min(steps),
                "Pattern ({}, {}) should have {} hits but got {}",
                hits,
                steps,
                hits.min(steps),
                events.len()
            );

            // Events should be within the bar (0.0 to 4.0 beats)
            for event in &events {
                assert!(
                    event.beat.0 >= 0.0 && event.beat.0 < 4.0,
                    "Event at beat {} is outside valid range for ({}, {})",
                    event.beat.0,
                    hits,
                    steps
                );
            }
        }
    }

    #[test]
    fn euclidean_rotation() {
        let base_pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 8);
        let rotated_pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 8).with_rotation(2);

        let base_events = base_pattern.events_for_bar(0);
        let rotated_events = rotated_pattern.events_for_bar(0);

        // Should have same number of hits
        assert_eq!(base_events.len(), rotated_events.len());

        // But different timing (unless the pattern is symmetric)
        let base_positions: Vec<f64> = base_events.iter().map(|e| e.beat.0).collect();
        let rotated_positions: Vec<f64> = rotated_events.iter().map(|e| e.beat.0).collect();

        println!("Base positions: {:?}", base_positions);
        println!("Rotated positions: {:?}", rotated_positions);
    }

    #[test]
    fn euclidean_velocity() {
        let pattern = EuclideanPattern::new(DrumVoice::Kick, 3, 8).with_velocity(100);
        let events = pattern.events_for_bar(0);

        for event in events {
            assert_eq!(event.velocity, 100);
            assert_eq!(event.voice, DrumVoice::Kick);
        }
    }

    #[test]
    fn can_quantize_beat() {
        let beat = Beat(1.73);
        let quantized = beat.quantize(0.5);
        assert_eq!(quantized.0, 1.5);
    }

    #[test]
    fn beat_humanization() {
        let beat = Beat(1.0);
        let humanized = beat.humanize(0.1);

        // Should be close to original but not exact
        assert!(humanized.0 >= 0.9 && humanized.0 <= 1.1);
    }
}
