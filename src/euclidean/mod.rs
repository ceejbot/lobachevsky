//! Euclidean rhythm generation. Two algorithms, because there were two
//! and I felt like implementing both of them.

use crate::LobachevskyError;

mod bjorklund;
mod breshenham;

pub use bjorklund::Bjorklund;
pub use breshenham::Breshenham;

pub trait EuclideanRhythm {
    /// Evenly distribute N pulses among the M steps. Beats are indicated by
    /// true.
    fn generate(steps: usize, pulses: usize) -> Result<Vec<bool>, LobachevskyError>;
}

pub fn print_pattern(pattern: &[bool]) {
    // Print as X and . for readability
    let visual: String = pattern.iter().map(|&x| if x { 'X' } else { '.' }).collect();

    log::info!("  Pattern: {}", visual);
    log::info!("  Debug:  {:?}", pattern);

    // Show pulse positions
    let positions: Vec<usize> = pattern
        .iter()
        .enumerate()
        .filter_map(|(i, &x)| if x { Some(i) } else { None })
        .collect();
    log::info!("  Pulses at positions: {:?}", positions);
}
