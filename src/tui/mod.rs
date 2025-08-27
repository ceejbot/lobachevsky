//! Terminal User Interface for Lobachevsky music generation
//!
//! Provides an interactive TUI for browsing patterns, harmonies, and generating
//! music without needing to memorize complex CLI commands.

pub mod app;
pub mod browser;
pub mod events;
pub mod preview;
pub mod ui;

pub use app::App;
pub use events::run_tui;

use crate::LobachevskyError;

/// Entry point for the TUI mode
pub fn start_tui() -> Result<(), LobachevskyError> {
    let mut terminal = ui::setup_terminal()?;
    let app_result = run_tui(&mut terminal);
    ui::restore_terminal(&mut terminal)?;
    app_result
}
