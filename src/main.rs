//! A command-line interface to generate midi files using the lobachevsky
//! library.

use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::{Parser, Subcommand};
use lobachevsky::core::{Chord, ChordQuality, Mode, PitchClass};
use lobachevsky::generation::{HexatonicCycle, HexatonicExplorer, MelodyGenerator, MelodyStrategy, ProgressionBuilder};
use lobachevsky::midi::Composition;
use lobachevsky::rhythm::GenrePatterns;
use lobachevsky::{LobachevskyError, Transform};
use miette::IntoDiagnostic;

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
        /// Style: 90s, 2000s, 2010s
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
            generate_progression(
                &start,
                &pattern,
                length,
                return_to_start,
                mode.as_deref(),
                tonic.as_deref(),
                &cli.output,
            );
        }
        Commands::Hexatonic { cycle, start } => {
            generate_hexatonic(&cycle, &start, &cli.output);
        }
        Commands::Ambient { style, bars, tempo } => {
            generate_ambient(&style, bars, tempo, &cli.output).into_diagnostic()?;
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
            generate_modal(&mode, &tonic, start.as_deref(), &pattern, length, analyze, &cli.output);
        }
        Commands::Extended {
            start,
            pattern,
            length,
            analyze,
        } => {
            generate_extended(&start, &pattern, length, analyze, &cli.output);
        }
    }
    Ok(())
}

fn parse_pitch_class(pc_str: &str) -> Option<PitchClass> {
    PitchClass::try_from(pc_str).ok()
}

fn parse_chord(chord_str: &str) -> Option<Chord> {
    let (root_str, quality) = ChordQuality::split_root_quality(chord_str);
    let root = PitchClass::try_from(root_str).ok()?;
    Some(Chord::new(root, quality))
}

fn parse_transforms(pattern_str: &str) -> Vec<Transform> {
    pattern_str
        .split(',')
        .filter_map(|s| Transform::try_from(s).ok())
        .collect()
}

fn parse_mode(mode_str: &str) -> Option<Mode> {
    Mode::try_from(mode_str).ok()
}

fn generate_progression(
    start: &str,
    pattern: &str,
    length: usize,
    return_to_start: bool,
    mode: Option<&str>,
    tonic: Option<&str>,
    output: &str,
) {
    let start_chord = parse_chord(start).unwrap_or(Chord::f_major());
    let transforms = parse_transforms(pattern);

    if transforms.is_empty() {
        eprintln!("Invalid transformation pattern");
        return;
    }

    let mut builder = ProgressionBuilder::new()
        .start(start_chord)
        .pattern(&transforms)
        .length(length);

    if return_to_start {
        builder = builder.with_return();
    }

    // Apply modal constraints if specified
    if let (Some(mode_str), Some(tonic_str)) = (mode, tonic) {
        if let (Some(parsed_mode), Some(tonic_pc)) = (parse_mode(mode_str), parse_pitch_class(tonic_str)) {
            builder = builder.with_mode(parsed_mode, tonic_pc);
            println!("Using modal constraint: {} {}", tonic_pc, parsed_mode);
        } else {
            eprintln!("Invalid mode or tonic specification");
            return;
        }
    } else if mode.is_some() || tonic.is_some() {
        eprintln!("Both mode and tonic must be specified for modal constraints");
        return;
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

    if let Err(e) = composition.save(output) {
        eprintln!("Failed to save MIDI file: {}", e);
    } else {
        println!("Saved to {}", output);
    }
}

fn generate_hexatonic(cycle_type: &str, start: &str, output: &str) {
    let start_chord = parse_chord(start).unwrap_or(Chord::c_major());

    let cycle = match cycle_type.to_lowercase().as_str() {
        "northern" => HexatonicCycle::Northern,
        "western" => HexatonicCycle::Western,
        "eastern" => HexatonicCycle::Eastern,
        _ => {
            eprintln!("Invalid cycle type. Use: northern, western, or eastern");
            return;
        }
    };

    let explorer = HexatonicExplorer::new();

    if let Some(chords) = explorer.get_cycle(cycle, start_chord) {
        println!("Hexatonic {} cycle starting from {}:", cycle_type, start_chord);
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

        if let Err(e) = composition.save(output) {
            eprintln!("Failed to save MIDI file: {}", e);
        } else {
            println!("Saved to {}", output);
        }
    } else {
        eprintln!("Could not generate hexatonic cycle");
    }
}

fn generate_ambient(style: &str, bars: usize, tempo: u16, output: &str) -> Result<(), LobachevskyError> {
    let pattern = match style {
        "90s" => GenrePatterns::ambient_90s(),
        "2000s" => GenrePatterns::microhouse_2000s(),
        "2010s" => GenrePatterns::euclidean_2010s(),
        _ => return Err(LobachevskyError::UnknownRhythmStyle(style.to_string())),
    };

    println!(
        "Generating {} ambient techno rhythm ({} bars at {} BPM)",
        style, bars, tempo
    );

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

fn generate_modal(
    mode_str: &str,
    tonic_str: &str,
    start: Option<&str>,
    pattern: &str,
    length: usize,
    analyze: bool,
    output: &str,
) {
    let Some(mode) = parse_mode(mode_str) else {
        eprintln!("Invalid mode: {}", mode_str);
        return;
    };

    let Some(tonic) = parse_pitch_class(tonic_str) else {
        eprintln!("Invalid tonic: {}", tonic_str);
        return;
    };

    let transforms = parse_transforms(pattern);
    if transforms.is_empty() {
        eprintln!("Invalid transformation pattern");
        return;
    }

    println!("Exploring {} {} mode with neo-Riemannian transformations", tonic, mode);

    // Create modal transformer
    use lobachevsky::ModalNeoRiemannian;
    let modal_transformer = ModalNeoRiemannian::new(mode, tonic);

    if analyze {
        let analysis = modal_transformer.analyze_mode();
        println!("\n=== Modal Analysis ===");
        println!("Mode: {} {}", tonic, mode);
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
    let start_chord = if let Some(start_str) = start {
        if let Some(chord) = parse_chord(start_str) {
            if modal_transformer.valid_chords().contains(&chord) {
                chord
            } else {
                eprintln!(
                    "Warning: {} is not in {} {} mode, using first chord of mode",
                    chord, tonic, mode
                );
                modal_transformer
                    .valid_chords()
                    .iter()
                    .next()
                    .copied()
                    .unwrap_or(Chord::c_major())
            }
        } else {
            eprintln!("Invalid starting chord, using first chord of mode");
            modal_transformer
                .valid_chords()
                .iter()
                .next()
                .copied()
                .unwrap_or(Chord::c_major())
        }
    } else {
        // Use the tonic chord of the mode
        mode.triads(tonic)[0]
    };

    println!("\nStarting from: {}", start_chord);

    // Generate modal progression using the transformer directly
    let progression = modal_transformer.apply_sequence(start_chord, &transforms[0..length.min(transforms.len())]);

    println!("\nModal progression ({} transformations):", progression.len() - 1);
    for (i, chord) in progression.iter().enumerate() {
        if i == 0 {
            println!("  Start: {}", chord);
        } else {
            println!(
                "  {}: {} (via {})",
                i,
                chord,
                match &transforms[(i - 1) % transforms.len()] {
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

    if let Err(e) = composition.save(output) {
        eprintln!("Failed to save MIDI file: {}", e);
    } else {
        println!("\nSaved modal exploration to {}", output);
    }
}

fn generate_extended(start: &str, pattern: &str, length: usize, analyze: bool, output: &str) {
    use lobachevsky::theory::{NeoRiemannian, Transformable};

    // Parse the starting extended chord
    let start_chord = match parse_chord(start) {
        Some(chord) => chord,
        None => {
            eprintln!("Invalid starting chord: {}", start);
            return;
        }
    };

    let transforms = parse_transforms(pattern);
    if transforms.is_empty() {
        eprintln!("Invalid transformation pattern");
        return;
    }

    println!("Exploring extended chord transformations");

    if analyze {
        println!("\n=== Extended Chord Analysis ===");
        println!("Starting chord: {}", start_chord);
        println!("Chord type: {:?}", start_chord.quality);
        println!("Is triad: {}", start_chord.quality.is_triad());
        println!("Is seventh chord: {}", start_chord.quality.is_seventh());
        println!("Is suspended: {}", start_chord.quality.is_suspended());

        if let Some(underlying) = start_chord.underlying_triad() {
            println!("Underlying triad: {}", underlying);
        }

        println!(
            "Supports basic transforms (P, R, L): {}",
            start_chord.supports_basic_transforms()
        );
        println!("Supports P3,0 transforms: {}", start_chord.supports_p3_transforms());
    }

    // Generate extended chord progression
    let transformer = NeoRiemannian::new();
    let progression = transformer.apply_sequence_any(start_chord, &transforms[0..length.min(transforms.len())]);

    println!(
        "\nExtended chord progression ({} transformations):",
        progression.len() - 1
    );
    for (i, chord) in progression.iter().enumerate() {
        if i == 0 {
            println!("  Start: {} ({:?})", chord, chord.quality);
        } else {
            let transform = &transforms[(i - 1) % transforms.len()];
            let transform_name = match transform {
                Transform::P => "P",
                Transform::R => "R",
                Transform::L => "L",
                Transform::Compound(_) => "compound",
            };

            if analyze {
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

    if let Err(e) = composition.save(output) {
        eprintln!("Failed to save MIDI file: {}", e);
    } else {
        println!("\nSaved extended chord exploration to {}", output);
    }
}
