//! Event handling and main TUI event loop

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use super::app::App;
use super::ui::{self, Tui};
use crate::LobachevskyError;

/// Main TUI event loop
pub fn run_tui(terminal: &mut Tui) -> Result<(), LobachevskyError> {
    let mut app = App::new()?;
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        // Render the current state
        terminal
            .draw(|frame| ui::render(frame, &app))
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
                handle_key_event(&mut app, key.code, key.modifiers)?;
            }
        }

        // Handle periodic updates
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            // Could add periodic updates here (file watching, etc.)
        }

        // Exit if requested
        if app.should_exit {
            break;
        }
    }

    Ok(())
}

/// Handle individual key events
fn handle_key_event(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> Result<(), LobachevskyError> {
    match key {
        // Global keys
        KeyCode::Char('q') => {
            app.should_exit = true;
        }

        KeyCode::Char('g') => {
            if modifiers.is_empty() {
                app.generate()?;
            }
        }

        // Tab navigation
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                app.previous_mode();
            } else {
                app.next_mode();
            }
        }

        // Arrow key navigation
        KeyCode::Up => {
            app.navigate_up();
        }

        KeyCode::Down => {
            app.navigate_down();
        }

        // Selection
        KeyCode::Enter => {
            app.select_current()?;
            if !app.param_editor.editing_mode {
                app.status = format!("Selected {} in {}", get_current_selection_name(app), app.mode_name());
            }
        }

        // Refresh (F5)
        KeyCode::F(5) => {
            refresh_browsers(app)?;
            app.status = "Refreshed file listings".to_string();
        }

        // Escape - cancel editing in params mode
        KeyCode::Esc => {
            if app.param_editor.editing_mode {
                app.param_editor.stop_editing();
                app.status = "Editing cancelled".to_string();
            } else {
                app.should_exit = true;
            }
        }

        // Backspace - for parameter editing
        KeyCode::Backspace => {
            if app.param_editor.editing_mode {
                app.handle_param_backspace();
            }
        }

        // Help (F1)
        KeyCode::F(1) => {
            app.status = "Help: ↑/↓=Navigate Tab=Categories Enter=Select Space=Preview G=Generate Q=Quit".to_string();
        }

        KeyCode::Char(c) => {
            // Handle character input for parameter editing
            if app.param_editor.editing_mode {
                app.handle_param_char(c);
            } else {
                // Handle other character-based commands
                handle_character_commands(app, c, modifiers)?;
            }
        }

        _ => {
            // Handle mode-specific keys
            handle_mode_specific_keys(app, key, modifiers)?;
        }
    }

    Ok(())
}

/// Handle keys specific to the current mode
fn handle_mode_specific_keys(app: &mut App, key: KeyCode, _modifiers: KeyModifiers) -> Result<(), LobachevskyError> {
    if app.mode == super::app::AppMode::Params {
        match key {
            KeyCode::Char('+') | KeyCode::Char('=') => {
                app.selections.bars = (app.selections.bars + 16).min(512);
                app.update_preview()?;
            }
            KeyCode::Char('-') => {
                app.selections.bars = (app.selections.bars.saturating_sub(16)).max(16);
                app.update_preview()?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handle character-specific mode keys
fn handle_mode_specific_keys_char(app: &mut App, c: char, _modifiers: KeyModifiers) -> Result<(), LobachevskyError> {
    if app.mode == super::app::AppMode::Params {
        match c {
            '+' | '=' => {
                app.selections.bars = (app.selections.bars + 16).min(512);
                app.update_preview()?;
            }
            '-' => {
                app.selections.bars = (app.selections.bars.saturating_sub(16)).max(16);
                app.update_preview()?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Get the name of the currently selected item for status display
fn get_current_selection_name(app: &App) -> String {
    match app.mode {
        super::app::AppMode::Drums => app.browsers.drums_browser.selected_file().unwrap_or("None").to_string(),
        super::app::AppMode::Bass => app.browsers.bass_browser.selected_file().unwrap_or("None").to_string(),
        super::app::AppMode::Harmony => app
            .browsers
            .harmony_browser
            .selected_file()
            .unwrap_or("None")
            .to_string(),
        super::app::AppMode::Melody => app.browsers.melody_browser.selected().unwrap_or("None").to_string(),
        super::app::AppMode::Groove => app.browsers.groove_browser.selected().unwrap_or("None").to_string(),
        _ => "".to_string(),
    }
}

/// Handle character-based commands (non-editing mode)
fn handle_character_commands(app: &mut App, c: char, modifiers: KeyModifiers) -> Result<(), LobachevskyError> {
    match c {
        // Numbers for quick category switching
        '1' => app.mode = super::app::AppMode::Drums,
        '2' => app.mode = super::app::AppMode::Bass,
        '3' => app.mode = super::app::AppMode::Harmony,
        '4' => app.mode = super::app::AppMode::Melody,
        '5' => app.mode = super::app::AppMode::Groove,
        '6' => app.mode = super::app::AppMode::Params,
        '7' => app.mode = super::app::AppMode::Preview,

        // Preview (space bar)
        ' ' => {
            app.status = "Preview not yet implemented".to_string();
        }

        // Help
        'h' => {
            app.status = "Help: ↑/↓=Navigate Tab=Categories Enter=Select Space=Preview G=Generate Q=Quit".to_string();
        }

        // Refresh (Ctrl+R)
        'r' if modifiers.contains(KeyModifiers::CONTROL) => {
            refresh_browsers(app)?;
            app.status = "Refreshed file listings".to_string();
        }

        _ => {
            // Handle mode-specific character input
            handle_mode_specific_keys_char(app, c, modifiers)?;
        }
    }
    Ok(())
}

/// Refresh all file browsers
fn refresh_browsers(app: &mut App) -> Result<(), LobachevskyError> {
    app.browsers.drums_browser.refresh()?;
    app.browsers.bass_browser.refresh()?;
    app.browsers.harmony_browser.refresh()?;
    Ok(())
}
