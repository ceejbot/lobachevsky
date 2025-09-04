//! UI rendering and terminal management for the In C thing

use std::io;

use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{Frame, Terminal};

use super::state::{GenerationStatus, SelectedField, State};
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
pub fn render(frame: &mut Frame<'_>, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(15),   // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    render_title(frame, chunks[0]);

    if state.show_help {
        render_help(frame, chunks[1]);
    } else {
        render_parameters(frame, chunks[1], state);
    }

    render_status_bar(frame, chunks[2], state);
}

fn render_title(frame: &mut Frame<'_>, area: Rect) {
    let title = Paragraph::new("In C Performance Generator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_parameters(frame: &mut Frame<'_>, area: Rect, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Basic settings title
            Constraint::Length(5), // Basic settings (3 params)
            Constraint::Length(3), // Performance settings title
            Constraint::Length(5), // Performance settings (4 params)
            Constraint::Min(3),    // Generation status
        ])
        .split(area);

    // Basic Settings
    let basic_title = Paragraph::new("Basic Settings")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
    frame.render_widget(basic_title, chunks[0]);

    let basic_params = render_basic_params(state);
    frame.render_widget(basic_params, chunks[1]);

    // Performance Settings
    let perf_title = Paragraph::new("Performance Settings")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
    frame.render_widget(perf_title, chunks[2]);

    let perf_params = render_performance_params(state);
    frame.render_widget(perf_params, chunks[3]);

    // Generation Status
    render_generation_status(frame, chunks[4], state);
}

fn render_basic_params(state: &State) -> List<'static> {
    let items = vec![
        ListItem::new(format_parameter_owned(
            "Number of Performers",
            format!(" {}", state.config.num_performers),
            state.selected_field == SelectedField::NumPerformers,
            "1-20 musicians",
        )),
        ListItem::new(format_parameter_owned(
            "Duration",
            format!(" {:.1} minutes", state.config.duration_minutes),
            state.selected_field == SelectedField::Duration,
            "5-60 minutes",
        )),
        ListItem::new(format_parameter_owned(
            "Tempo",
            format!(" {} BPM", state.config.tempo),
            state.selected_field == SelectedField::Tempo,
            "60-180 BPM",
        )),
        ListItem::new(format_parameter_owned(
            "Include Pulse Track",
            if state.config.include_pulse {
                " Yes".to_string()
            } else {
                " No".to_string()
            },
            state.selected_field == SelectedField::IncludePulse,
            "Eighth-note pulse",
        )),
    ];

    List::new(items).block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM))
}

fn render_performance_params(state: &State) -> List<'static> {
    let items = vec![
        ListItem::new(format_slider_owned(
            "Variation",
            state.config.variation,
            state.selected_field == SelectedField::Variation,
            "Performance diversity",
        )),
        ListItem::new(format_slider_owned(
            "Canon Probability",
            state.config.canon_probability,
            state.selected_field == SelectedField::CanonProbability,
            "Chance of echo voices",
        )),
        ListItem::new(format_slider_owned(
            "Timing Flexibility",
            state.config.timing_flex,
            state.selected_field == SelectedField::TimingFlex,
            "Rhythmic variation",
        )),
    ];

    List::new(items).block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM))
}

fn format_parameter_owned(name: &str, value: String, selected: bool, hint: &str) -> Line<'static> {
    let style = if selected {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let indicator = if selected { "> " } else { "  " };

    Line::from(vec![
        Span::styled(indicator.to_string(), style),
        Span::styled(format!("{:20}", name), style),
        Span::styled(format!("{:15}", value), Style::default().fg(Color::Cyan)),
        Span::styled(format!(" ({})", hint), Style::default().fg(Color::DarkGray)),
    ])
}

fn format_slider_owned(name: &str, value: f32, selected: bool, hint: &str) -> Line<'static> {
    let style = if selected {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let indicator = if selected { "> " } else { "  " };
    let percentage = (value * 100.0) as u8;
    let bar_width = 20;
    let filled = ((value * bar_width as f32) as usize).min(bar_width);
    let bar = format!(
        "[{}{}] {:3}%",
        "=".repeat(filled),
        " ".repeat(bar_width - filled),
        percentage
    );

    Line::from(vec![
        Span::styled(indicator.to_string(), style),
        Span::styled(format!("{:20}", name), style),
        Span::styled(bar, Style::default().fg(Color::Cyan)),
        Span::styled(format!(" ({})", hint), Style::default().fg(Color::DarkGray)),
    ])
}

fn render_generation_status(frame: &mut Frame<'_>, area: Rect, state: &State) {
    let (text, style) = match &state.generation_status {
        GenerationStatus::Idle => (
            "Ready to generate. Press 'g' to create MIDI file.".to_string(),
            Style::default().fg(Color::Gray),
        ),
        GenerationStatus::Generating => (
            "Generating performance...".to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::SLOW_BLINK),
        ),
        GenerationStatus::Success(msg) => (msg.clone(), Style::default().fg(Color::Green)),
        GenerationStatus::Error(msg) => (format!("Error: {}", msg), Style::default().fg(Color::Red)),
    };

    let status = Paragraph::new(text)
        .style(style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);
}

fn render_help(frame: &mut Frame<'_>, area: Rect) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab/Shift+Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" - Navigate between parameters"),
        ]),
        Line::from(vec![
            Span::styled("↑/↓ or ←/→", Style::default().fg(Color::Cyan)),
            Span::raw(" - Adjust selected value"),
        ]),
        Line::from(vec![
            Span::styled("Shift + Arrows", Style::default().fg(Color::Cyan)),
            Span::raw(" - Large adjustments"),
        ]),
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Cyan)),
            Span::raw(" - Toggle boolean values"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("g", Style::default().fg(Color::Green)),
            Span::raw(" - Generate MIDI file"),
        ]),
        Line::from(vec![
            Span::styled("r", Style::default().fg(Color::Magenta)),
            Span::raw(" - Reset to defaults"),
        ]),
        Line::from(vec![
            Span::styled("F1", Style::default().fg(Color::Blue)),
            Span::raw(" - Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::raw(" - Quit"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame<'_>, area: Rect, state: &State) {
    let help_hint = if state.show_help { "F1: Close Help" } else { "F1: Help" };

    let shortcuts = format!(" {} | g: Generate | r: Reset | q: Quit ", help_hint);

    let status_bar = Paragraph::new(shortcuts)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status_bar, area);
}
