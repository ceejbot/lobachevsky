//! This is the other way to generate Euclidean rhythms.
//! It produces slightly different results but is simpler.

use super::EuclideanRhythm;
use crate::LobachevskyError;

pub struct Breshenham {}

impl EuclideanRhythm for Breshenham {
    fn generate(steps: usize, pulses: usize) -> Result<Vec<bool>, LobachevskyError> {
        if pulses > steps {
            return Err(LobachevskyError::EuclideanInputs { steps, pulses });
        }

        // Handle edge cases
        if pulses == 0 {
            return Ok(vec![false; steps]);
        }
        if pulses == steps {
            return Ok(vec![true; steps]);
        }

        let mut pattern = vec![false; steps];

        // Use Bresenham-like algorithm to distribute hits evenly
        let mut error = 0i32;
        let threshold = steps as i32;

        #[allow(clippy::needless_range_loop)]
        for i in 0..steps {
            error += pulses as i32;
            if error >= threshold {
                pattern[i] = true;
                error -= steps as i32;
            }
        }

        // Need to start with pulse aka beat
        if let Some(idx) = pattern.iter().position(|b| *b) {
            pattern.rotate_left(idx);
        }

        Ok(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_euclidean_patterns() {
        // Classic Euclidean rhythm examples

        // 5 pulses in 8 steps - Cuban tresillo-like
        let result = Breshenham::generate(8, 5).expect("test pattern should be okay");
        // Should distribute 5 pulses as evenly as possible across 8 steps
        assert_eq!(result.len(), 8);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 5);

        // 3 pulses in 8 steps - basic pattern
        let result = Breshenham::generate(8, 3).expect("test pattern should be okay");
        assert_eq!(result.len(), 8);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 3);

        // 4 pulses in 16 steps - standard 4/4 kick pattern
        let result = Breshenham::generate(16, 4).expect("test pattern should be okay");
        assert_eq!(result.len(), 16);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 4);
    }

    #[test]
    fn edge_cases_are_good() {
        // All pulses
        let result = Breshenham::generate(4, 4).expect("test pattern should be okay");
        assert_eq!(result, vec![true, true, true, true]);

        // No pulses
        let result = Breshenham::generate(4, 0).expect("test pattern should be okay");
        assert_eq!(result, vec![false, false, false, false]);

        // Single pulse
        let result = Breshenham::generate(4, 1).expect("test pattern should be okay");
        assert_eq!(result, vec![true, false, false, false]);
        assert_eq!(result.len(), 4);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 1);
        assert!(result[0]); // Should start with the pulse due to rotation

        // Single step
        let result = Breshenham::generate(1, 1).expect("test pattern should be okay");
        assert_eq!(result, vec![true]);

        let result = Breshenham::generate(1, 0).expect("test pattern should be okay");
        assert_eq!(result, vec![false]);
    }

    #[test]
    fn can_handle_invalid_input() {
        // More pulses than steps should fail
        assert!(Breshenham::generate(4, 5).is_err());
        assert!(Breshenham::generate(8, 10).is_err());
    }

    #[test]
    fn known_patterns_work_properly() {
        // Test against known Euclidean rhythms

        // E(3,8) - classic tresillo pattern (rotated to start with 1)
        let result = Breshenham::generate(8, 3).expect("test pattern should be okay");
        assert_eq!(result.len(), 8);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 3);
        // Should have good distribution - no more than 3 consecutive 0s
        let max_consecutive_zeros = max_consecutive_falses(&result);
        assert!(max_consecutive_zeros <= 3);

        // E(5,8) - Cinquillo pattern
        let result = Breshenham::generate(8, 5).expect("test pattern should be okay");
        assert_eq!(result.len(), 8);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 5);

        // E(7,12) - West African bell pattern
        let result = Breshenham::generate(12, 7).expect("test pattern should be okay");
        assert_eq!(result.len(), 12);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 7);
    }

    #[test]
    fn patterns_start_with_pulse() {
        // Due to rotation, patterns should start with a pulse (1) when there are pulses
        let result = Breshenham::generate(8, 3).expect("test pattern should be okay");
        assert!(result[0]);

        let result = Breshenham::generate(16, 5).expect("test pattern should be okay");
        assert!(result[0]);

        // Exception: no pulses means no starting pulse
        let result = Breshenham::generate(8, 0).expect("test pattern should be okay");
        assert!(!result[0]);
    }

    #[test]
    fn bresenham_specific_patterns() {
        // Test some patterns that might differ from Bjorklund

        // E(5,13) - prime number case
        let result = Breshenham::generate(13, 5).expect("test pattern should be okay");
        assert_eq!(result.len(), 13);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 5);
        assert!(result[0]); // Should start with pulse

        // E(3,7) - another prime case
        let result = Breshenham::generate(7, 3).expect("test pattern should be okay");
        assert_eq!(result.len(), 7);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 3);
        assert!(result[0]); // Should start with pulse

        // E(2,5) - minimal case
        let result = Breshenham::generate(5, 2).expect("test pattern should be okay");
        assert_eq!(result.len(), 5);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 2);
        assert!(result[0]); // Should start with pulse
    }

    /// Helper function to find maximum consecutive zeros in a pattern
    fn max_consecutive_falses(pattern: &[bool]) -> usize {
        let mut max_zeros = 0;
        let mut current_zeros = 0;

        for &value in pattern {
            if !value {
                current_zeros += 1;
                max_zeros = max_zeros.max(current_zeros);
            } else {
                current_zeros = 0;
            }
        }

        max_zeros
    }
}
