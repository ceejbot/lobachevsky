//! UI rendering and terminal management for the TUI

use std::io;

use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap};
use ratatui::{Frame, Terminal};

use super::app::{App, AppMode};
use crate::LobachevskyError;

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

/// Setup the terminal for TUI mode
pub fn setup_terminal() -> Result<Tui, LobachevskyError> {
    enable_raw_mode().map_err(|e| LobachevskyError::TuiError {
        message: format!("Failed to enable raw mode: {}", e),
    })?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| LobachevskyError::TuiError {
        message: format!("Failed to enter alternate screen: {}", e),
    })?;

    Terminal::new(CrosstermBackend::new(stdout)).map_err(|e| LobachevskyError::TuiError {
        message: format!("Failed to create terminal: {}", e),
    })
}

/// Restore the terminal to normal mode
pub fn restore_terminal(terminal: &mut Tui) -> Result<(), LobachevskyError> {
    disable_raw_mode().map_err(|e| LobachevskyError::TuiError {
        message: format!("Failed to disable raw mode: {}", e),
    })?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| LobachevskyError::TuiError {
        message: format!("Failed to leave alternate screen: {}", e),
    })?;

    Ok(())
}

/// Main UI rendering function
pub fn render(frame: &mut Frame<'_>, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Tab bar
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    render_header(frame, main_layout[0], app);
    render_content(frame, main_layout[1], app);
    render_tabs(frame, main_layout[2], app);
    render_status(frame, main_layout[3], app);
}

/// Render the header with current selections
fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let header_text = format!("🎵 Lobachevsky Music Generator - {}", app.preview.summary_line());

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Current Selection"))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(header, area);
}

/// Render the main content area based on current mode
fn render_content(frame: &mut Frame<'_>, area: Rect, app: &App) {
    match app.mode {
        AppMode::Drums => render_drums_browser(frame, area, app),
        AppMode::Bass => render_bass_browser(frame, area, app),
        AppMode::Harmony => render_harmony_browser(frame, area, app),
        AppMode::Melody => render_melody_selector(frame, area, app),
        AppMode::Groove => render_groove_selector(frame, area, app),
        AppMode::Params => render_params_editor(frame, area, app),
        AppMode::Preview => render_preview_screen(frame, area, app),
    }
}

/// Render drums pattern browser
fn render_drums_browser(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let browser_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left panel: File list
    let browser = &app.browsers.drums_browser;
    let items: Vec<ListItem<'_>> = browser
        .files
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let filename = &entry.name;
            let is_cursor = i == browser.selected_index;
            let is_selected = app.selections.drums_pattern.as_ref() == Some(filename);

            let display_name = browser.display_name(entry);

            // Create visual indicator with cursor and selection feedback
            let prefix = match (is_cursor, is_selected) {
                (true, true) => "► ✓",   // Cursor on selected item
                (true, false) => "►  ",  // Cursor only
                (false, true) => "  ✓",  // Selected but not cursor
                (false, false) => "   ", // Neither
            };

            let full_display = format!("{} {}", prefix, display_name);

            let style = if is_selected && is_cursor {
                Style::default().bg(Color::Blue).fg(Color::Yellow) // Selected + cursor
            } else if is_selected {
                Style::default().fg(Color::Green) // Selected
            } else if is_cursor {
                Style::default().bg(Color::Blue).fg(Color::White) // Cursor only
            } else {
                Style::default()
            };

            ListItem::new(full_display).style(style)
        })
        .collect();

    let file_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Drum Patterns ({} available)", browser.file_count())),
        )
        .highlight_style(Style::default().bg(Color::Blue));

    frame.render_widget(file_list, browser_layout[0]);

    // Right panel: Pattern details
    if let Some(entry) = browser.selected_entry() {
        render_pattern_details(frame, browser_layout[1], entry, browser);
    } else {
        let empty = Paragraph::new("No patterns available")
            .block(Block::default().borders(Borders::ALL).title("Pattern Details"))
            .alignment(Alignment::Center);
        frame.render_widget(empty, browser_layout[1]);
    }
}

/// Render bass pattern browser
fn render_bass_browser(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let browser_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left panel: File list
    let browser = &app.browsers.bass_browser;
    let items: Vec<ListItem<'_>> = browser
        .files
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let filename = &entry.name;
            let is_cursor = i == browser.selected_index;
            let is_selected = app.selections.bass_pattern.as_ref() == Some(filename);

            let display_name = browser.display_name(entry);

            // Create visual indicator with cursor and selection feedback
            let prefix = match (is_cursor, is_selected) {
                (true, true) => "▶ ✓",   // Cursor on selected item
                (true, false) => "▶  ",  // Cursor only
                (false, true) => "  ✓",  // Selected but not cursor
                (false, false) => "   ", // Neither
            };

            let full_display = format!("{} {}", prefix, display_name);

            let style = if is_selected && is_cursor {
                Style::default().bg(Color::Red).fg(Color::Yellow) // Selected + cursor
            } else if is_selected {
                Style::default().fg(Color::Green) // Selected
            } else if is_cursor {
                Style::default().bg(Color::Red).fg(Color::White) // Cursor only
            } else {
                Style::default()
            };

            ListItem::new(full_display).style(style)
        })
        .collect();

    let file_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Bass Patterns ({} available)", browser.file_count())),
        )
        .highlight_style(Style::default().bg(Color::Red));

    frame.render_widget(file_list, browser_layout[0]);

    // Right panel: Pattern details
    if let Some(entry) = browser.selected_entry() {
        render_pattern_details(frame, browser_layout[1], entry, browser);
    } else {
        let empty = Paragraph::new("No bass patterns available")
            .block(Block::default().borders(Borders::ALL).title("Pattern Details"))
            .alignment(Alignment::Center);
        frame.render_widget(empty, browser_layout[1]);
    }
}

/// Render harmony pattern browser
fn render_harmony_browser(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let browser_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left panel: File list
    let browser = &app.browsers.harmony_browser;
    let items: Vec<ListItem<'_>> = browser
        .files
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let filename = &entry.name;
            let is_cursor = i == browser.selected_index;
            let is_selected = app.selections.harmony_pattern.as_ref() == Some(filename);

            let display_name = browser.display_name(entry);

            // Create visual indicator with cursor and selection feedback
            let prefix = match (is_cursor, is_selected) {
                (true, true) => "► ✓",   // Cursor on selected item
                (true, false) => "►  ",  // Cursor only
                (false, true) => "  ✓",  // Selected but not cursor
                (false, false) => "   ", // Neither
            };

            let full_display = format!("{} {}", prefix, display_name);

            let style = if is_selected && is_cursor {
                Style::default().bg(Color::Green).fg(Color::Yellow) // Selected + cursor
            } else if is_selected {
                Style::default().fg(Color::Green) // Selected
            } else if is_cursor {
                Style::default().bg(Color::Green).fg(Color::White) // Cursor only
            } else {
                Style::default()
            };

            ListItem::new(full_display).style(style)
        })
        .collect();

    let file_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Harmony Patterns ({} available)", browser.file_count())),
        )
        .highlight_style(Style::default().bg(Color::Green));

    frame.render_widget(file_list, browser_layout[0]);

    // Right panel: Harmony details
    if let Some(entry) = browser.selected_entry() {
        render_pattern_details(frame, browser_layout[1], entry, browser);
    } else {
        let empty = Paragraph::new("No harmony patterns available")
            .block(Block::default().borders(Borders::ALL).title("Harmony Details"))
            .alignment(Alignment::Center);
        frame.render_widget(empty, browser_layout[1]);
    }
}

/// Render melody strategy selector
fn render_melody_selector(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let browser = &app.browsers.melody_browser;

    let items: Vec<ListItem<'_>> = browser
        .strategies
        .iter()
        .enumerate()
        .map(|(i, strategy)| {
            let is_cursor = i == browser.selected_index;
            let is_selected = app.selections.melody_strategy.as_ref() == Some(strategy);

            let description = match strategy.as_str() {
                "lead_synth" => "Soaring lead synthesizer lines with filter sweeps",
                "arpeggiated" => "Fast arpeggiated sequences, classic electronic style",
                "rhythmic_stabs" => "Punchy rhythmic chord stabs and accents",
                "textural_pads" => "Ambient pad textures and atmospheric layers",
                "pluck_sequence" => "Bright pluck sequences with gate effects",
                "bass_lead" => "Low-register lead lines with sub-bass character",
                _ => "Electronic melody strategy",
            };

            // Create visual indicator with cursor and selection feedback
            let prefix = match (is_cursor, is_selected) {
                (true, true) => "► ✓",   // Cursor on selected item
                (true, false) => "►  ",  // Cursor only
                (false, true) => "  ✓",  // Selected but not cursor
                (false, false) => "   ", // Neither
            };

            let full_display = format!("{} {} - {}", prefix, strategy, description);

            let style = if is_selected && is_cursor {
                Style::default().bg(Color::Magenta).fg(Color::Yellow) // Selected + cursor
            } else if is_selected {
                Style::default().fg(Color::Green) // Selected
            } else if is_cursor {
                Style::default().bg(Color::Magenta).fg(Color::White) // Cursor only
            } else {
                Style::default()
            };

            ListItem::new(full_display).style(style)
        })
        .collect();

    let melody_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Melody Strategies"))
        .highlight_style(Style::default().bg(Color::Magenta));

    frame.render_widget(melody_list, area);
}

/// Render groove/humanization selector
fn render_groove_selector(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let browser = &app.browsers.groove_browser;

    let items: Vec<ListItem<'_>> = browser
        .groove_types
        .iter()
        .enumerate()
        .map(|(i, groove_type)| {
            let is_cursor = i == browser.selected_index;
            let is_selected = app.selections.groove_settings.as_ref() == Some(groove_type);

            let description = match groove_type.as_str() {
                "human" => "Natural timing variations for human feel",
                "tight" => "Precise timing with minimal variation",
                "loose" => "Relaxed timing with noticeable swing",
                "swing" => "Classic jazz swing feel",
                "shuffle" => "Triplet-based shuffle groove",
                _ => "Unknown groove type",
            };

            // Create visual indicator with cursor and selection feedback
            let prefix = match (is_cursor, is_selected) {
                (true, true) => "▶ ✓",   // Cursor on selected item
                (true, false) => "▶  ",  // Cursor only
                (false, true) => "  ✓",  // Selected but not cursor
                (false, false) => "   ", // Neither
            };

            let full_display = format!("{} {} - {}", prefix, groove_type, description);

            let style = if is_selected && is_cursor {
                Style::default().bg(Color::Yellow).fg(Color::Black) // Selected + cursor
            } else if is_selected {
                Style::default().fg(Color::Green) // Selected
            } else if is_cursor {
                Style::default().bg(Color::Yellow).fg(Color::White) // Cursor only
            } else {
                Style::default()
            };

            ListItem::new(full_display).style(style)
        })
        .collect();

    let groove_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Groove & Humanization"))
        .highlight_style(Style::default().bg(Color::Yellow));

    frame.render_widget(groove_list, area);
}

/// Render parameter editor
fn render_params_editor(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let param_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Instructions
            Constraint::Min(6),    // Parameter list
        ])
        .split(area);

    // Instructions
    let instructions = if app.param_editor.editing_mode {
        "Enter: Apply | Esc: Cancel | Type to edit"
    } else {
        "↑/↓: Navigate | Enter: Edit parameter"
    };

    let instruction_widget = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(instruction_widget, param_layout[0]);

    // Parameter list
    let params = [
        ("Bars", app.selections.bars.to_string()),
        (
            "Tempo",
            app.selections
                .tempo
                .map(|t| t.to_string())
                .unwrap_or("Auto".to_string()),
        ),
        ("Output File", app.selections.output_file.clone()),
        ("Key Signature", app.selections.key_signature.clone()),
        ("Starting Chord", app.selections.starting_chord.clone()),
        (
            "Bass Strategy",
            app.selections
                .bass_strategy
                .clone()
                .unwrap_or_else(|| "root".to_string()),
        ),
    ];

    let items: Vec<ListItem<'_>> = params
        .iter()
        .enumerate()
        .map(|(i, (name, value))| {
            let is_selected = i == app.param_editor.selected_param;
            let is_editing = is_selected && app.param_editor.editing_mode;

            let display_value = if is_editing {
                format!("{}: {}█", name, app.param_editor.edit_buffer) // Cursor indicator
            } else {
                format!("{}: {}", name, value)
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            let full_display = format!("{}{}", prefix, display_value);

            let style = if is_editing {
                Style::default().bg(Color::Yellow).fg(Color::Black) // Editing mode
            } else if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White) // Selected
            } else {
                Style::default()
            };

            ListItem::new(full_display).style(style)
        })
        .collect();

    let param_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Generation Parameters"))
        .highlight_style(Style::default().bg(Color::Blue));

    frame.render_widget(param_list, param_layout[1]);
}

/// Render preview screen
fn render_preview_screen(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let preview_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Summary
            Constraint::Min(4),    // Details
            Constraint::Length(4), // Warnings
        ])
        .split(area);

    // Summary section
    let summary_text = format!(
        "Ready to Generate:\n{}\n\nDuration: {} | Tracks: {}",
        app.preview.description,
        app.preview.duration_string(),
        app.preview.track_count
    );

    let summary = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Generation Summary"))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(summary, preview_layout[0]);

    // Details section
    let details_text = format!(
        "Drums: {}\nBass: {}\nHarmony: {}\nMelody: {}\nGroove: {}\nBass Strategy: {}\nKey: {} | Start: {}\nBars: {} | Tempo: {} BPM",
        app.selections.drums_pattern.as_deref().unwrap_or("None"),
        app.selections.bass_pattern.as_deref().unwrap_or("None"),
        app.selections.harmony_pattern.as_deref().unwrap_or("None"),
        app.selections.melody_strategy.as_deref().unwrap_or("None"),
        app.selections.groove_settings.as_deref().unwrap_or("None"),
        app.selections.bass_strategy.as_deref().unwrap_or("root"),
        app.selections.key_signature,
        app.selections.starting_chord,
        app.selections.bars,
        app.preview.tempo
    );

    let details = Paragraph::new(details_text)
        .block(Block::default().borders(Borders::ALL).title("Detailed Settings"))
        .wrap(Wrap { trim: true });

    frame.render_widget(details, preview_layout[1]);

    // Warnings section
    let warnings_text = if app.preview.warnings.is_empty() {
        "✓ Ready to generate!".to_string()
    } else {
        app.preview.warnings.join("\n")
    };

    let warnings_style = if app.preview.warnings.is_empty() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let warnings = Paragraph::new(warnings_text)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .style(warnings_style)
        .wrap(Wrap { trim: true });

    frame.render_widget(warnings, preview_layout[2]);
}

/// Render pattern details in the right panel
fn render_pattern_details(
    frame: &mut Frame<'_>,
    area: Rect,
    entry: &super::browser::FileEntry,
    browser: &super::browser::FileBrowser,
) {
    let title = format!("Details: {}", browser.display_name(entry));

    let mut lines = vec![Line::from(vec![
        Span::styled("Name: ", Style::default().fg(Color::Gray)),
        Span::raw(browser.display_name(entry)),
    ])];

    if let Some(tempo) = browser.tempo_hint(entry) {
        lines.push(Line::from(vec![
            Span::styled("Tempo: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{} BPM", tempo)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Description:",
        Style::default().fg(Color::Gray),
    )]));

    let description = browser.description(entry);
    for line in description.lines() {
        lines.push(Line::from(line));
    }

    let details = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true });

    frame.render_widget(details, area);
}

/// Render bottom tab bar
fn render_tabs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let tab_titles = vec!["Drums", "Bass", "Harmony", "Melody", "Groove", "Params", "Preview"];

    let selected_tab = match app.mode {
        AppMode::Drums => 0,
        AppMode::Bass => 1,
        AppMode::Harmony => 2,
        AppMode::Melody => 3,
        AppMode::Groove => 4,
        AppMode::Params => 5,
        AppMode::Preview => 6,
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .select(selected_tab);

    frame.render_widget(tabs, area);
}

/// Render status bar with help and current info
fn render_status(frame: &mut Frame<'_>, area: Rect, app: &App) {
    // Create a layout with two sections: selections and help
    let status_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Left side: Current selections summary
    let drums_status = app
        .selections
        .drums_pattern
        .as_ref()
        .map(|d| format!("D:{}", d))
        .unwrap_or_else(|| "D:None".to_string());

    let bass_status = app
        .selections
        .bass_pattern
        .as_ref()
        .map(|b| format!("B:{}", b))
        .unwrap_or_else(|| "B:None".to_string());

    let harmony_status = app
        .selections
        .harmony_pattern
        .as_ref()
        .map(|h| format!("H:{}", h))
        .unwrap_or_else(|| "H:None".to_string());

    let melody_status = app
        .selections
        .melody_strategy
        .as_ref()
        .map(|m| format!("M:{}", m))
        .unwrap_or_else(|| "M:mixed".to_string());

    let selections_text = format!(
        "{} | {} | {} | {} | {}bars",
        drums_status, bass_status, harmony_status, melody_status, app.selections.bars
    );

    let selections = Paragraph::new(selections_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Left);

    // Right side: Help text
    let help_text = "↑/↓: Navigate | Tab: Categories | Enter: Select | G: Generate | Q: Quit";

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Right);

    frame.render_widget(selections, status_layout[0]);
    frame.render_widget(help, status_layout[1]);
}
