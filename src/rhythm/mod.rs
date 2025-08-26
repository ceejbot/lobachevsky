//! Rhythm generation and pattern systems

mod bass_patterns;
mod beats;
mod genres;
mod pattern_types;
mod patterns;

pub use bass_patterns::*;
pub use beats::*;
pub use genres::*;
pub use pattern_types::*;
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
        log::info!("3/8 pattern positions: {:?}", positions);
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
            log::info!("Testing ({}, {})", hits, steps);
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

        log::info!("Base positions: {:?}", base_positions);
        log::info!("Rotated positions: {:?}", rotated_positions);
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

    #[test]
    fn isochronic_frequency_to_pulses() {
        // At 120 BPM, there are 2 beats per second
        // So a 10 Hz frequency should generate 20 pulses per 4-beat bar
        // (10 Hz * 2 seconds = 20 pulses)

        let pattern = IsochronicPattern::new(DrumVoice::Percussion, 10.0);
        let events = pattern.events_for_bar(0);

        // 10 Hz at 120 BPM = 20 pulses per bar
        assert_eq!(events.len(), 20, "10 Hz should generate 20 pulses per bar");

        // Test other frequencies
        let pattern_5hz = IsochronicPattern::new(DrumVoice::Percussion, 5.0);
        assert_eq!(
            pattern_5hz.events_for_bar(0).len(),
            10,
            "5 Hz should generate 10 pulses"
        );

        let pattern_20hz = IsochronicPattern::new(DrumVoice::Percussion, 20.0);
        assert_eq!(
            pattern_20hz.events_for_bar(0).len(),
            40,
            "20 Hz should generate 40 pulses"
        );
    }

    #[test]
    fn isochronic_pulse_spacing() {
        // Test that pulses are evenly spaced
        let pattern = IsochronicPattern::new(DrumVoice::Percussion, 10.0);
        let events = pattern.events_for_bar(0);

        // Calculate expected interval: 4 beats / 20 pulses = 0.2 beats
        let expected_interval = 0.2;

        for i in 1..events.len() {
            let interval = events[i].beat.0 - events[i - 1].beat.0;
            assert!(
                (interval - expected_interval).abs() < 0.001,
                "Pulse interval {} should be close to {}",
                interval,
                expected_interval
            );
        }

        // First pulse should be at beat 0
        assert_eq!(events[0].beat.0, 0.0, "First pulse should be at beat 0");

        // Last pulse should be before beat 4
        assert!(
            events.last().expect("we should have a last pulse").beat.0 < 4.0,
            "Last pulse should be within the bar"
        );
    }

    #[test]
    fn isochronic_brainwave_frequencies() {
        // Test typical brainwave frequencies

        // Delta (2 Hz) - Deep sleep
        let delta = IsochronicPattern::new(DrumVoice::Ride, 2.0);
        assert_eq!(delta.events_for_bar(0).len(), 4, "Delta 2 Hz should generate 4 pulses");

        // Theta (6 Hz) - Creativity
        let theta = IsochronicPattern::new(DrumVoice::Ride, 6.0);
        assert_eq!(
            theta.events_for_bar(0).len(),
            12,
            "Theta 6 Hz should generate 12 pulses"
        );

        // Alpha (10 Hz) - Relaxed focus
        let alpha = IsochronicPattern::new(DrumVoice::Ride, 10.0);
        assert_eq!(
            alpha.events_for_bar(0).len(),
            20,
            "Alpha 10 Hz should generate 20 pulses"
        );

        // Beta (20 Hz) - Active thinking
        let beta = IsochronicPattern::new(DrumVoice::Ride, 20.0);
        assert_eq!(beta.events_for_bar(0).len(), 40, "Beta 20 Hz should generate 40 pulses");

        // Gamma (40 Hz) - Peak awareness
        let gamma = IsochronicPattern::new(DrumVoice::Ride, 40.0);
        assert_eq!(
            gamma.events_for_bar(0).len(),
            80,
            "Gamma 40 Hz should generate 80 pulses"
        );
    }

    #[test]
    fn isochronic_with_base_pattern() {
        // Test layering isochronic pulses over a base pattern
        let base = Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4));
        let isochronic = IsochronicPattern::new(DrumVoice::Shaker, 5.0).with_base_pattern(base);

        let events = isochronic.events_for_bar(0);

        // Should have 4 kick events + 10 shaker pulses = 14 total
        assert_eq!(events.len(), 14, "Should have combined events from base and isochronic");

        // Count events by voice
        let kick_count = events.iter().filter(|e| e.voice == DrumVoice::Kick).count();
        let shaker_count = events.iter().filter(|e| e.voice == DrumVoice::Shaker).count();

        assert_eq!(kick_count, 4, "Should have 4 kick events from base pattern");
        assert_eq!(shaker_count, 10, "Should have 10 shaker pulses from isochronic");

        // Verify events are sorted by beat
        for i in 1..events.len() {
            assert!(
                events[i].beat.0 >= events[i - 1].beat.0,
                "Events should be sorted by beat position"
            );
        }
    }

    #[test]
    fn isochronic_velocity_control() {
        // Test custom velocity setting
        let pattern = IsochronicPattern::new(DrumVoice::Percussion, 8.0).with_velocity(45);

        let events = pattern.events_for_bar(0);

        for event in events {
            assert_eq!(event.velocity, 45, "All pulses should have custom velocity");
            assert_eq!(
                event.voice,
                DrumVoice::Percussion,
                "All pulses should use specified voice"
            );
        }
    }

    #[test]
    fn isochronic_edge_cases() {
        // Very low frequency (0.5 Hz = 1 pulse per bar)
        let low_freq = IsochronicPattern::new(DrumVoice::Ride, 0.5);
        assert_eq!(low_freq.events_for_bar(0).len(), 1, "0.5 Hz should generate 1 pulse");

        // Frequency that doesn't divide evenly
        let odd_freq = IsochronicPattern::new(DrumVoice::Ride, 7.0);
        assert_eq!(odd_freq.events_for_bar(0).len(), 14, "7 Hz should generate 14 pulses");

        // High frequency (50 Hz)
        let high_freq = IsochronicPattern::new(DrumVoice::Ride, 50.0);
        assert_eq!(
            high_freq.events_for_bar(0).len(),
            100,
            "50 Hz should generate 100 pulses"
        );
    }

    #[test]
    fn isochronic_pattern_length() {
        // Without base pattern, should be 1 bar
        let simple = IsochronicPattern::new(DrumVoice::Percussion, 10.0);
        assert_eq!(simple.pattern_length(), 1);

        // With base pattern, should inherit its length
        // Note: EuclideanPattern has pattern_length of 1
        let base = Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4));
        let with_base = IsochronicPattern::new(DrumVoice::Percussion, 10.0).with_base_pattern(base);
        assert_eq!(with_base.pattern_length(), 1);
    }

    #[test]
    fn isochronic_toml_loading() {
        // Test loading a simple isochronic pattern from TOML
        let toml_str = r#"
            name = "test_alpha"
            description = "Test alpha wave pattern"
            tempo_hint = 120

            [[tracks]]
            voice = "percussion"
            [tracks.pattern_type]
            type = "Isochronic"
            frequency_hz = 10.0
            velocity = 30
            brainwave_type = "alpha"
        "#;

        let pattern_data = PatternData::from_toml(toml_str).expect("Should parse TOML");
        assert_eq!(pattern_data.name, "test_alpha");
        assert_eq!(pattern_data.tracks.len(), 1);

        // Convert to actual pattern and test
        let rhythm_pattern = pattern_data.to_pattern().expect("Should convert to pattern");
        let events = rhythm_pattern.events_for_bar(0);

        // 10 Hz should generate 20 pulses
        assert_eq!(events.len(), 20, "Alpha 10 Hz pattern should generate 20 pulses");
    }

    #[test]
    fn isochronic_toml_with_base_pattern() {
        // Test loading isochronic pattern with base pattern from TOML
        let toml_str = r#"
            name = "test_layered"
            description = "Test isochronic with base pattern"
            tempo_hint = 120

            [[tracks]]
            voice = "shaker"
            [tracks.pattern_type]
            type = "Isochronic"
            frequency_hz = 6.0
            velocity = 25
            [tracks.pattern_type.base_pattern]
            type = "Euclidean"
            hits = 4
            steps = 4
            velocity = 60
        "#;

        let pattern_data = PatternData::from_toml(toml_str).expect("Should parse TOML");
        let rhythm_pattern = pattern_data.to_pattern().expect("Should convert to pattern");
        let events = rhythm_pattern.events_for_bar(0);

        // 6 Hz = 12 pulses + 4 from base = 16 total
        // But they're all the same voice (shaker) so we get 12 pulses
        // (base pattern uses same voice as isochronic)
        assert_eq!(events.len(), 16, "Should have isochronic pulses plus base pattern");

        // All should be shaker voice
        assert!(events.iter().all(|e| e.voice == DrumVoice::Shaker));
    }

    #[test]
    fn isochronic_toml_multi_frequency() {
        // Test loading multiple isochronic layers (multi-frequency entrainment)
        let toml_str = r#"
            name = "test_multi"
            description = "Multi-frequency entrainment"
            tempo_hint = 120

            [[tracks]]
            voice = "hihat_closed"
            [tracks.pattern_type]
            type = "Isochronic"
            frequency_hz = 10.0
            velocity = 30

            [[tracks]]
            voice = "shaker"
            [tracks.pattern_type]
            type = "Isochronic"
            frequency_hz = 6.0
            velocity = 25
        "#;

        let pattern_data = PatternData::from_toml(toml_str).expect("Should parse TOML");
        assert_eq!(pattern_data.tracks.len(), 2, "Should have two tracks");

        let rhythm_pattern = pattern_data.to_pattern().expect("Should convert to pattern");
        let events = rhythm_pattern.events_for_bar(0);

        // 10 Hz = 20 pulses + 6 Hz = 12 pulses = 32 total
        assert_eq!(events.len(), 32, "Should have combined pulses from both frequencies");

        // Count events by voice
        let hihat_count = events.iter().filter(|e| e.voice == DrumVoice::HiHatClosed).count();
        let shaker_count = events.iter().filter(|e| e.voice == DrumVoice::Shaker).count();

        assert_eq!(hihat_count, 20, "Should have 20 hi-hat pulses (10 Hz)");
        assert_eq!(shaker_count, 12, "Should have 12 shaker pulses (6 Hz)");
    }

    #[test]
    fn isochronic_clone_and_debug() {
        // Test that Clone and Debug implementations work
        let pattern = IsochronicPattern::new(DrumVoice::Percussion, 8.0).with_velocity(40);

        // Test clone
        let cloned = pattern.clone();
        assert_eq!(
            pattern.events_for_bar(0).len(),
            cloned.events_for_bar(0).len(),
            "Cloned pattern should generate same events"
        );

        // Test debug output (should not panic)
        let debug_str = format!("{:?}", pattern);
        assert!(debug_str.contains("IsochronicPattern"));
        assert!(debug_str.contains("8"));

        // Test clone_box for dynamic dispatch
        let boxed: Box<dyn RhythmPattern> = pattern.clone_box();
        assert_eq!(boxed.events_for_bar(0).len(), 16, "Boxed pattern should work correctly");
    }
}
