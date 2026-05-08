//! Kinds of melodies.

mod call_response;
mod markov;
mod riley;
mod strategy;

pub use call_response::{CallResponseGenerator, CallResponsePhrase, CallResponseType, Phrase};
pub use markov::{MarkovChain, MarkovStats, MelodyMarkov};
pub use riley::{InCPatterns, RileyNote, RileyPattern, RileyPatternData, SelfHarmony};
pub use strategy::{MelodyGenerator, MelodyStrategy};
