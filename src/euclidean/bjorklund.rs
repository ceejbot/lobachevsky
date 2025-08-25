//! An implementation of Bjorklund's algorithm in Rust.
//! There are two references I consulted:
//! Godfried Toussaint's original paper: http://cgm.cs.mcgill.ca/~godfried/publications/banff.pdf
//! Brian House's python implementation: https://github.com/brianhouse/bjorklund

use super::EuclideanRhythm;
use crate::LobachevskyError;

pub struct Bjorklund {}

impl EuclideanRhythm for Bjorklund {
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

        let mut temp_pattern: Vec<bool> = Vec::with_capacity(steps);
        let mut counts: Vec<usize> = Vec::with_capacity(64);
        let mut remainders: Vec<usize> = Vec::with_capacity(64);
        let mut divisor = steps - pulses;

        remainders.push(pulses);
        let mut level = 0;

        loop {
            if remainders[level] == 0 {
                break;
            }
            counts.push(divisor / remainders[level]);
            remainders.push(divisor % remainders[level]);
            divisor = remainders[level];
            level += 1;
            if remainders[level] <= 1 {
                break;
            }
        }
        counts.push(divisor);

        // Build the pattern by appending to temp_pattern
        fn build_pattern(pattern: &mut Vec<bool>, counts: &[usize], remainders: &[usize], level: isize) {
            if level == -1 {
                pattern.push(false);
            } else if level == -2 {
                pattern.push(true);
            } else {
                let current_level = level as usize;
                for _i in 0..counts[current_level] {
                    build_pattern(pattern, counts, remainders, level - 1);
                }
                if remainders[current_level] != 0 {
                    build_pattern(pattern, counts, remainders, level - 2);
                }
            }
        }

        build_pattern(&mut temp_pattern, &counts, &remainders, level as isize);

        // Find the first beat and rotate pattern so it starts there
        if let Some(idx) = temp_pattern.iter().position(|&t| t) {
            temp_pattern.rotate_left(idx);
        }
        Ok(temp_pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_euclidean_patterns() {
        // Classic Euclidean rhythm examples

        // 5 pulses in 8 steps - Cuban tresillo-like
        let result = Bjorklund::generate(8, 5).expect("test pattern should be okay");
        // Should distribute 5 pulses as evenly as possible across 8 steps
        assert_eq!(result.len(), 8);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 5);

        // 3 pulses in 8 steps - basic pattern
        let result = Bjorklund::generate(8, 3).expect("test pattern should be okay");
        assert_eq!(result.len(), 8);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 3);

        // 4 pulses in 16 steps - standard 4/4 kick pattern
        let result = Bjorklund::generate(16, 4).expect("test pattern should be okay");
        assert_eq!(result.len(), 16);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 4);
    }

    #[test]
    fn edge_cases_are_good() {
        // All pulses
        let result = Bjorklund::generate(4, 4).expect("test pattern should be okay");
        assert_eq!(result, vec![true, true, true, true]);

        // No pulses
        let result = Bjorklund::generate(4, 0).expect("test pattern should be okay");
        assert_eq!(result, vec![false, false, false, false]);

        // Single pulse
        let result = Bjorklund::generate(4, 1).expect("test pattern should be okay");
        assert_eq!(result, vec![true, false, false, false]);
        assert_eq!(result.len(), 4);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 1);
        assert!(result[0]); // Should start with the pulse due to rotation

        // Single step
        let result = Bjorklund::generate(1, 1).expect("test pattern should be okay");
        assert_eq!(result, vec![true]);

        let result = Bjorklund::generate(1, 0).expect("test pattern should be okay");
        assert_eq!(result, vec![false]);
    }

    #[test]
    fn can_handle_invalid_input() {
        // More pulses than steps should fail
        assert!(Bjorklund::generate(4, 5).is_err());
        assert!(Bjorklund::generate(8, 10).is_err());
    }

    #[test]
    fn known_patterns_work_properly() {
        // Test against known Euclidean rhythms

        // E(3,8) - classic tresillo pattern (rotated to start with 1)
        let result = Bjorklund::generate(8, 3).expect("test pattern should be okay");
        assert_eq!(result.len(), 8);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 3);
        // Should have good distribution - no more than 3 consecutive 0s
        let max_consecutive_zeros = max_consecutive_falses(&result);
        assert!(max_consecutive_zeros <= 3);

        // E(5,8) - Cinquillo pattern
        let result = Bjorklund::generate(8, 5).expect("test pattern should be okay");
        assert_eq!(result.len(), 8);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 5);

        // E(7,12) - West African bell pattern
        let result = Bjorklund::generate(12, 7).expect("test pattern should be okay");
        assert_eq!(result.len(), 12);
        assert_eq!(result.iter().fold(0, |acc, b| { if *b { acc + 1 } else { acc } }), 7);
    }

    #[test]
    fn patterns_start_with_pulse() {
        // Due to rotation, patterns should start with a pulse (1) when there are pulses
        let result = Bjorklund::generate(8, 3).expect("test pattern should be okay");
        assert!(result[0]);

        let result = Bjorklund::generate(16, 5).expect("test pattern should be okay");
        assert!(result[0]);

        // Exception: no pulses means no starting pulse
        let result = Bjorklund::generate(8, 0).expect("test pattern should be okay");
        assert!(!result[0]);
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
