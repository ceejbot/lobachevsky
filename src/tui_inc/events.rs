//! Main event loop & handlers.

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use super::state::{SelectedField, State};
use super::ui::{self, Tui};
use crate::LobachevskyError;

/// Main TUI event loop
pub fn run_tui(terminal: &mut Tui) -> Result<(), LobachevskyError> {
    let mut state = State::new()?;
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        // Render the current state
        terminal
            .draw(|frame| ui::render(frame, &state))
            .map_err(|e| LobachevskyError::TuiError {
                message: format!("Failed to draw: {}", e),
            })?;

        // Handle events
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout).map_err(|e| LobachevskyError::TuiError {
            message: format!("Failed to poll events: {}", e),
        })? && let Event::Key(key) = event::read().map_err(|e| LobachevskyError::TuiError {
            message: format!("Failed to read event: {}", e),
        })? {
            // Handle cross-platform key event differences
            if key.kind == KeyEventKind::Press {
                handle_key_event(&mut state, key.code, key.modifiers)?;
            }
        }

        // Handle periodic updates
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            // Could add periodic updates here (file watching, etc.)
        }

        // Exit if requested
        if state.should_exit {
            break;
        }
    }

    Ok(())
}

/// Handle individual key events
fn handle_key_event(state: &mut State, key: KeyCode, modifiers: KeyModifiers) -> Result<(), LobachevskyError> {
    // Always allow quit and help toggle
    match key {
        KeyCode::Char('q') => {
            state.should_exit = true;
            return Ok(());
        }
        KeyCode::F(1) => {
            state.show_help = !state.show_help;
            return Ok(());
        }
        _ => {}
    }

    // Don't process other keys if help is showing
    if state.show_help {
        return Ok(());
    }

    match key {
        // Generate MIDI
        KeyCode::Char('g') => {
            if modifiers.is_empty() {
                state.generate()?;
            }
        }

        // Reset to defaults
        KeyCode::Char('r') => {
            if modifiers.is_empty() {
                state.reset_to_defaults();
            }
        }

        // Tab navigation between fields
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                state.selected_field = state.selected_field.previous();
            } else {
                state.selected_field = state.selected_field.next();
            }
        }

        // The arrow keys move up and down through fields
        KeyCode::Up => {
            state.selected_field = state.selected_field.previous();
        }

        KeyCode::Down => {
            state.selected_field = state.selected_field.next();
        }

        // Left and right key navigation for adjusting values
        KeyCode::Right => {
            state.increment_field(modifiers.contains(KeyModifiers::SHIFT));
        }

        KeyCode::Left => {
            state.decrement_field(modifiers.contains(KeyModifiers::SHIFT));
        }

        // Space to toggle boolean fields
        KeyCode::Char(' ') => {
            if state.selected_field == SelectedField::IncludePulse {
                state.toggle_boolean_field();
            }
        }

        // Enter could also toggle boolean or confirm current value
        KeyCode::Enter => {
            if state.selected_field == SelectedField::IncludePulse {
                state.toggle_boolean_field();
            }
        }

        _ => {}
    }

    Ok(())
}
