//! A command-line interface to generate midi files using the lobachevsky
//! library.

use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::{Parser, Subcommand};
use lobachevsky::core::Chord;
use lobachevsky::generation::ProgressionBuilder;
use lobachevsky::generators::*;
use lobachevsky::melody::{MelodyGenerator, MelodyStrategy};
use lobachevsky::midi::Composition;
use lobachevsky::rhythm::GenrePatterns;
use lobachevsky::{LobachevskyError, Transform};

#[derive(Parser)]
#[command(author, version, about, long_about = None, styles = v3_styles(), max_term_width = 100)]
struct Cli {
    /// Quiet output
    #[arg(long, short, default_value_t = false, global = true)]
    quiet: bool,
    /// Verbose output
    #[arg(long, short, default_value_t = false, global = true)]
    verbose: bool,
    /// What kind of musical exploration to generate
    #[command(subcommand)]
    command: Commands,
    /// Where to store the generated MIDI
    #[arg(default_value = "generated_progression.mid", global = true)]
    output: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a neo-Riemannian progression
    Progression {
        /// Starting chord (e.g., "F", "Am", "C#m")
        #[arg(short, long, default_value = "F")]
        start: String,

        /// Transformation pattern (P, R, L)
        #[arg(short, long, default_value = "P,R,L")]
        pattern: String,

        /// Number of chords in progression
        #[arg(short, long, default_value_t = 8)]
        length: usize,

        /// Return to starting chord
        #[arg(short, long)]
        return_to_start: bool,

        /// Modal constraint (e.g., "lydian", "dorian", "mixolydian")
        #[arg(short, long)]
        mode: Option<String>,

        /// Modal tonic (e.g., "C", "F#", "Bb") - required if mode is specified
        #[arg(short, long)]
        tonic: Option<String>,
    },

    /// Explore hexatonic cycles
    Hexatonic {
        /// Cycle type: northern, western, eastern
        #[arg(short, long, default_value = "northern")]
        cycle: String,

        /// Starting chord
        #[arg(short, long, default_value = "C")]
        start: String,
    },

    /// Generate ambient techno composition
    Ambient {
        /// Style: 90s, 2000s, 2010s, detroit, berlin, minimal, acid, deep
        #[arg(short, long, default_value = "90s")]
        style: String,

        /// Number of bars
        #[arg(short, long, default_value_t = 64)]
        bars: usize,

        /// BPM
        #[arg(short, long, default_value_t = 110)]
        tempo: u16,
    },

    /// Generate a complete composition with harmony, rhythm, and melody
    Compose {
        /// Number of bars
        #[arg(short, long, default_value_t = 64)]
        bars: usize,

        /// BPM
        #[arg(short, long, default_value_t = 110)]
        tempo: u16,
    },

    /// Explore modal neo-Riemannian transformations
    Modal {
        /// Mode (ionian, dorian, phrygian, lydian, mixolydian, aeolian,
        /// locrian)
        #[arg(short, long, default_value = "lydian")]
        mode: String,

        /// Tonic pitch class (C, D, E, F, G, A, B, etc.)
        #[arg(short, long, default_value = "C")]
        tonic: String,

        /// Starting chord within the mode
        #[arg(short, long)]
        start: Option<String>,

        /// Transformation pattern
        #[arg(short, long, default_value = "P,L,R")]
        pattern: String,

        /// Number of transformations
        #[arg(short, long, default_value_t = 12)]
        length: usize,

        /// Show mode analysis
        #[arg(short, long)]
        analyze: bool,
    },

    /// Explore extended chord transformations (seventh chords, suspended
    /// chords)
    Extended {
        /// Starting extended chord (e.g., "Cmaj7", "Am7", "Fsus4", "G7")
        #[arg(short, long, default_value = "Cmaj7")]
        start: String,

        /// Transformation pattern (P, R, L for basic; P3 for seventh chord P3,0
        /// transforms)
        #[arg(short, long, default_value = "P,R,L")]
        pattern: String,

        /// Number of chords in progression
        #[arg(short, long, default_value_t = 8)]
        length: usize,

        /// Show chord analysis (underlying triads, chord types)
        #[arg(short, long)]
        analyze: bool,
    },

    /// Launch interactive TUI for music generation
    Tui,

    /// Generate a complete algorithmic composition
    Generate {
        /// Rhythm pattern name from library (e.g., "deep_hypnotic",
        /// "coding_flow")
        #[arg(short, long)]
        rhythm: Option<String>,

        /// Harmonic pattern file (TOML) or inline start chord
        #[arg(short = 'H', long)]
        harmony: Option<String>,

        /// Starting chord if not using harmony file (e.g., "Am", "C#m", "F")
        #[arg(short = 's', long, default_value = "Am")]
        start: String,

        /// Transformation pattern if not using harmony file (P, R, L)
        #[arg(short = 'p', long, default_value = "P,L,R")]
        pattern: String,

        /// Mode constraint (e.g., "dorian", "lydian")
        #[arg(short, long)]
        mode: Option<String>,

        /// Modal tonic if mode is specified (e.g., "C", "D", "F#")
        #[arg(short, long)]
        tonic: Option<String>,

        /// Melody generation strategy (chord_tones, arpeggio, stepwise, mixed)
        #[arg(short = 'M', long, default_value = "mixed")]
        melody: String,

        /// Number of bars (will be aligned to 16/32 bar multiples)
        #[arg(short, long, default_value_t = 32)]
        bars: usize,

        /// Tempo in BPM (defaults to pattern tempo_hint if available, otherwise
        /// 120)
        #[arg(short = 'T', long)]
        tempo: Option<u16>,

        /// Notes per chord for melody
        #[arg(short, long, default_value_t = 16)]
        notes_per_chord: usize,

        /// Force return to starting chord for alignment
        #[arg(short = 'R', long)]
        return_to_start: bool,
    },
}

/// I like my clap help styled the old way.
fn v3_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

fn main() -> miette::Result<()> {
    miette::set_panic_hook();
    let cli = Cli::parse();

    // Disable logging for TUI mode to prevent output interference
    let level = match &cli.command {
        Commands::Tui => log::LevelFilter::Off, // Silent for TUI
        _ => {
            if cli.quiet {
                log::LevelFilter::Warn
            } else if cli.verbose {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            }
        }
    };

    let config = lovely_env_logger::Config {
        with_system_timestamp: false,
        reltime: false,
        short_levels: false,
        with_file_name: false,
        with_line_number: false,
        with_padding: true,
    };
    lovely_env_logger::formatted_builder(config).filter(None, level).init();

    match cli.command {
        Commands::Progression {
            start,
            pattern,
            length,
            return_to_start,
            mode,
            tonic,
        } => {
            let input = progression::ProgressionInput::from_cli(
                &start,
                &pattern,
                length,
                return_to_start,
                mode.as_deref(),
                tonic.as_deref(),
            )?;
            progression::generate_progression(input, &cli.output)?;
        }
        Commands::Hexatonic { cycle, start } => {
            let input = hexatonic::HexatonicInput::from_cli(&cycle, &start)?;
            hexatonic::generate_hexatonic(input, &cli.output)?;
        }
        Commands::Ambient { style, bars, tempo } => {
            generate_ambient(&style, bars, tempo, &cli.output)?;
        }
        Commands::Compose { bars, tempo } => {
            generate_composition(bars, tempo, &cli.output);
        }
        Commands::Modal {
            mode,
            tonic,
            start,
            pattern,
            length,
            analyze,
        } => {
            let input = modal::ModalInput::from_cli(&mode, &tonic, start.as_deref(), &pattern, length, analyze)?;
            modal::generate_modal(input, &cli.output)?;
        }
        Commands::Extended {
            start,
            pattern,
            length,
            analyze,
        } => {
            let input = extended::ExtendedInput::from_cli(&start, &pattern, length, analyze)?;
            extended::generate_extended(input, &cli.output)?;
        }
        Commands::Tui => {
            use lobachevsky::tui;
            tui::start_tui()?;
        }
        Commands::Generate {
            rhythm,
            harmony,
            start,
            pattern,
            mode,
            tonic,
            melody,
            bars,
            tempo,
            notes_per_chord,
            return_to_start,
        } => {
            // Resolve tempo with hints
            let resolved_tempo = algorithmic::resolve_tempo_with_hints(
                tempo,
                rhythm.as_deref(),
                harmony.as_deref(),
                &start,
                &pattern,
                mode.as_deref(),
                tonic.as_deref(),
                return_to_start,
            )?;

            let input = algorithmic::GenerateInput::from_cli(
                rhythm.as_deref(),
                harmony.as_deref(),
                &start,
                &pattern,
                mode.as_deref(),
                tonic.as_deref(),
                &melody,
                bars,
                resolved_tempo,
                notes_per_chord,
                return_to_start,
            )?;
            algorithmic::generate_algorithmic(input, bars, Some(resolved_tempo), &cli.output)?;
        }
    }
    Ok(())
}

fn generate_ambient(style: &str, bars: usize, tempo: u16, output: &str) -> Result<(), LobachevskyError> {
    let pattern = match style {
        "90s" => GenrePatterns::ambient_90s(),
        "2000s" => GenrePatterns::microhouse_2000s(),
        "2010s" => GenrePatterns::euclidean_2010s(),
        "detroit" => GenrePatterns::detroit_techno(),
        "berlin" => GenrePatterns::berlin_dub_techno(),
        "minimal" => GenrePatterns::minimal_berlin(),
        "acid" => GenrePatterns::acid_minimal(),
        "deep" => GenrePatterns::deep_minimal(),
        _ => {
            return Err(LobachevskyError::UnknownRhythmStyle {
                style: style.to_string(),
            });
        }
    };

    log::info!("Generating {} techno rhythm ({} bars at {} BPM)", style, bars, tempo);

    let mut composition = Composition::new(tempo);
    composition.add_rhythm_track(pattern.as_ref(), bars);

    composition.save(output)?;
    log::info!("Saved to {}", output);
    Ok(())
}

fn generate_composition(bars: usize, tempo: u16, output: &str) {
    log::info!("Generating complete composition ({} bars at {} BPM)", bars, tempo);

    // Generate neo-Riemannian progression
    let progression = ProgressionBuilder::new()
        .start(Chord::f_major())
        .pattern(&[Transform::P, Transform::R, Transform::L])
        .length(bars / 4)
        .with_return()
        .build();

    log::info!("Chord progression:");
    for (i, chord) in progression.iter().enumerate() {
        log::info!("  Bar {}: {}", i * 4 + 1, chord);
    }

    // Generate melody
    let melody_gen = MelodyGenerator::new(MelodyStrategy::Mixed)
        .with_octave(5)
        .with_note_duration(0.25);

    let melody = melody_gen.generate(&progression, 16); // 16 notes per chord

    // Create composition
    let mut composition = Composition::new(tempo);

    // Add harmony track
    composition.add_harmony_track(&progression, 3, 4);

    // Add rhythm track
    let rhythm = GenrePatterns::ambient_90s();
    composition.add_rhythm_track(rhythm.as_ref(), bars);

    // Add melody track
    composition.add_melody_track(&melody, 1);

    if let Err(e) = composition.save(output) {
        log::error!("Failed to save MIDI file: {}", e);
    } else {
        log::info!("Saved complete composition to {}", output);
    }
}
