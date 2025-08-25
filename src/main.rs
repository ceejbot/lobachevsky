//! A command-line interface to generate midi files using the lobachevsky
//! library.

use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::{Parser, Subcommand};
use lobachevsky::cli_inputs::{ExtendedInput, GenerateInput, HexatonicInput, ModalInput, ProgressionInput};
use lobachevsky::core::Chord;
use lobachevsky::generation::{HexatonicExplorer, ProgressionBuilder};
use lobachevsky::generators::AlgorithmicComposition;
use lobachevsky::melody::{MelodyGenerator, MelodyStrategy};
use lobachevsky::midi::Composition;
use lobachevsky::rhythm::{EuclideanPattern, GenrePatterns};
use lobachevsky::{LobachevskyError, Transform};

#[derive(Parser)]
#[command(author, version, about, long_about = None, styles = v3_styles(), max_term_width = 100)]
struct Cli {
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

        /// Tempo in BPM
        #[arg(short = 'T', long, default_value_t = 120)]
        tempo: u16,

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

    match cli.command {
        Commands::Progression {
            start,
            pattern,
            length,
            return_to_start,
            mode,
            tonic,
        } => {
            let input = ProgressionInput::from_cli(
                &start,
                &pattern,
                length,
                return_to_start,
                mode.as_deref(),
                tonic.as_deref(),
            )?;
            generate_progression(input, &cli.output)?;
        }
        Commands::Hexatonic { cycle, start } => {
            let input = HexatonicInput::from_cli(&cycle, &start)?;
            generate_hexatonic(input, &cli.output)?;
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
            let input = ModalInput::from_cli(&mode, &tonic, start.as_deref(), &pattern, length, analyze)?;
            generate_modal(input, &cli.output)?;
        }
        Commands::Extended {
            start,
            pattern,
            length,
            analyze,
        } => {
            let input = ExtendedInput::from_cli(&start, &pattern, length, analyze)?;
            generate_extended(input, &cli.output)?;
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
            let input = GenerateInput::from_cli(
                rhythm.as_deref(),
                harmony.as_deref(),
                &start,
                &pattern,
                mode.as_deref(),
                tonic.as_deref(),
                &melody,
                bars,
                tempo,
                notes_per_chord,
                return_to_start,
            )?;
            generate_algorithmic(input, bars, tempo, &cli.output)?;
        }
    }
    Ok(())
}

fn generate_progression(input: ProgressionInput, output: &str) -> Result<(), LobachevskyError> {
    let mut builder = ProgressionBuilder::new()
        .start(input.start_chord)
        .pattern(&input.transforms)
        .length(input.length);

    if input.return_to_start {
        builder = builder.with_return();
    }

    // Apply modal constraints if specified
    if let (Some(mode), Some(tonic)) = (input.mode, input.tonic) {
        builder = builder.with_mode(mode, tonic);
        println!("Using modal constraint: {} {}", tonic, mode);
    }

    let progression = builder.build();

    // Print the progression
    println!("Generated progression:");
    for (i, chord) in progression.iter().enumerate() {
        println!("  {}: {}", i + 1, chord);
    }

    // Generate MIDI
    let mut composition = Composition::new(110);
    composition.add_harmony_track(&progression, 4, 4);

    composition.save(output)?;
    println!("Saved to {}", output);
    Ok(())
}

fn generate_hexatonic(input: HexatonicInput, output: &str) -> Result<(), LobachevskyError> {
    let explorer = HexatonicExplorer::new();

    if let Some(chords) = explorer.get_cycle(input.cycle, input.start_chord) {
        println!("Hexatonic cycle starting from {}:", input.start_chord);
        for (i, chord) in chords.iter().enumerate() {
            println!("  {}: {}", i + 1, chord);
        }

        // Generate MIDI - repeat the cycle several times
        let mut full_progression = Vec::new();
        for _ in 0..4 {
            full_progression.extend_from_slice(&chords);
        }

        let mut composition = Composition::new(110);
        composition.add_harmony_track(&full_progression, 4, 2);

        composition.save(output)?;
        println!("Saved to {}", output);
        Ok(())
    } else {
        Err(LobachevskyError::ParseError {
            message: "Could not generate hexatonic cycle".to_string(),
        })
    }
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

    println!("Generating {} techno rhythm ({} bars at {} BPM)", style, bars, tempo);

    let mut composition = Composition::new(tempo);
    composition.add_rhythm_track(pattern.as_ref(), bars);

    composition.save(output)?;
    println!("Saved to {}", output);
    Ok(())
}

fn generate_composition(bars: usize, tempo: u16, output: &str) {
    println!("Generating complete composition ({} bars at {} BPM)", bars, tempo);

    // Generate neo-Riemannian progression
    let progression = ProgressionBuilder::new()
        .start(Chord::f_major())
        .pattern(&[Transform::P, Transform::R, Transform::L])
        .length(bars / 4)
        .with_return()
        .build();

    println!("Chord progression:");
    for (i, chord) in progression.iter().enumerate() {
        println!("  Bar {}: {}", i * 4 + 1, chord);
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
        eprintln!("Failed to save MIDI file: {}", e);
    } else {
        println!("Saved complete composition to {}", output);
    }
}

fn generate_modal(input: ModalInput, output: &str) -> Result<(), LobachevskyError> {
    println!(
        "Exploring {} {} mode with neo-Riemannian transformations",
        input.tonic, input.mode
    );

    // Create modal transformer
    use lobachevsky::ModalNeoRiemannian;
    let modal_transformer = ModalNeoRiemannian::new(input.mode, input.tonic);

    if input.analyze {
        let analysis = modal_transformer.analyze_mode();
        println!("\n=== Modal Analysis ===");
        println!("Mode: {} {}", input.tonic, input.mode);
        println!("Available chords:");
        for (i, chord) in analysis.chords.iter().enumerate() {
            let degree = match i {
                0 => "I",
                1 => "ii",
                2 => "iii",
                3 => "IV",
                4 => "V",
                5 => "vi",
                6 => "vii°",
                _ => "?",
            };
            println!("  {}: {}", degree, chord);
        }

        if !analysis.characteristic_chords.is_empty() {
            println!("Characteristic chords:");
            for chord in &analysis.characteristic_chords {
                println!("  {}", chord);
            }
        }
    }

    // Determine starting chord
    let start_chord = if let Some(chord) = input.start_chord {
        if modal_transformer.valid_chords().contains(&chord) {
            chord
        } else {
            eprintln!(
                "Warning: {} is not in {} {} mode, using first chord of mode",
                chord, input.tonic, input.mode
            );
            modal_transformer
                .valid_chords()
                .iter()
                .next()
                .copied()
                .unwrap_or(Chord::c_major())
        }
    } else {
        // Use the tonic chord of the mode
        input.mode.triads(input.tonic)[0]
    };

    println!("\nStarting from: {}", start_chord);

    // Generate modal progression using the transformer directly
    let progression = modal_transformer.apply_sequence(
        start_chord,
        &input.transforms[0..input.length.min(input.transforms.len())],
    );

    println!("\nModal progression ({} transformations):", progression.len() - 1);
    for (i, chord) in progression.iter().enumerate() {
        if i == 0 {
            println!("  Start: {}", chord);
        } else {
            println!(
                "  {}: {} (via {})",
                i,
                chord,
                match &input.transforms[(i - 1) % input.transforms.len()] {
                    Transform::P => "P",
                    Transform::R => "R",
                    Transform::L => "L",
                    Transform::Compound(_) => "compound",
                }
            );
        }
    }

    // Generate MIDI
    let mut composition = Composition::new(110);
    composition.add_harmony_track(&progression, 4, 2);

    composition.save(output)?;
    println!("\nSaved modal exploration to {}", output);
    Ok(())
}

fn generate_extended(input: ExtendedInput, output: &str) -> Result<(), LobachevskyError> {
    use lobachevsky::theory::{NeoRiemannian, Transformable};

    println!("Exploring extended chord transformations");

    if input.analyze {
        println!("\n=== Extended Chord Analysis ===");
        println!("Starting chord: {}", input.start_chord);
        println!("Chord type: {:?}", input.start_chord.quality);
        println!("Is triad: {}", input.start_chord.quality.is_triad());
        println!("Is seventh chord: {}", input.start_chord.quality.is_seventh());
        println!("Is suspended: {}", input.start_chord.quality.is_suspended());

        if let Some(underlying) = input.start_chord.underlying_triad() {
            println!("Underlying triad: {}", underlying);
        }

        println!(
            "Supports basic transforms (P, R, L): {}",
            input.start_chord.supports_basic_transforms()
        );
        println!(
            "Supports P3,0 transforms: {}",
            input.start_chord.supports_p3_transforms()
        );
    }

    // Generate extended chord progression
    let transformer = NeoRiemannian::new();
    let progression = transformer.apply_sequence_any(
        input.start_chord,
        &input.transforms[0..input.length.min(input.transforms.len())],
    );

    println!(
        "\nExtended chord progression ({} transformations):",
        progression.len() - 1
    );
    for (i, chord) in progression.iter().enumerate() {
        if i == 0 {
            println!("  Start: {} ({:?})", chord, chord.quality);
        } else {
            let transform = &input.transforms[(i - 1) % input.transforms.len()];
            let transform_name = match transform {
                Transform::P => "P",
                Transform::R => "R",
                Transform::L => "L",
                Transform::Compound(_) => "compound",
            };

            if input.analyze {
                println!("  {}: {} ({:?}) (via {})", i, chord, chord.quality, transform_name);
                if let Some(underlying) = chord.underlying_triad() {
                    println!("      Underlying: {}", underlying);
                }
            } else {
                println!("  {}: {} (via {})", i, chord, transform_name);
            }
        }
    }

    // Generate MIDI
    let mut composition = Composition::new(110);
    composition.add_harmony_track(&progression, 4, 2);

    composition.save(output)?;
    println!("\nSaved extended chord exploration to {}", output);
    Ok(())
}

fn generate_algorithmic(input: GenerateInput, bars: usize, tempo: u16, output: &str) -> Result<(), LobachevskyError> {
    // Load or create rhythm pattern
    let rhythm = if let Some(rhythm_name) = &input.rhythm_name {
        AlgorithmicComposition::load_rhythm_pattern(rhythm_name)?
    } else {
        Box::new(EuclideanPattern::default())
    };

    lobachevsky::generators::algorithmic_composition(
        input.harmony, rhythm, input.melody_strategy, bars, tempo, input.notes_per_chord, output,
    )
}
