//! Terminal User Interface for configuring a performance of Terry Riley's "In
//! C".
//!
//! Provides an interactive TUI for conducting the algorithmic performers.

mod events;
mod state;
mod ui;

pub use state::State;

use crate::LobachevskyError;

/// Entry point for the TUI mode
pub fn start() -> Result<(), LobachevskyError> {
    let mut terminal = ui::setup_terminal()?;
    let app_result = events::run_tui(&mut terminal);
    ui::restore_terminal(&mut terminal)?;
    app_result
}
