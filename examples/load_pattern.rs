//! Example of loading rhythm patterns from TOML files

use std::path::Path;

use lobachevsky::library::{Library, PatternLibrary};
use lobachevsky::midi::Composition;
use lobachevsky::rhythm::PatternData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a single pattern from TOML
    let toml_content = std::fs::read_to_string("patterns/uk_garage.toml")?;
    let pattern_data = PatternData::from_toml(&toml_content)?;

    println!("Loaded pattern: {}", pattern_data.name);
    if let Some(desc) = &pattern_data.description {
        println!("Description: {}", desc);
    }
    if let Some(tempo) = pattern_data.tempo_hint {
        println!("Suggested tempo: {} BPM", tempo);
    }

    // Convert to RhythmPattern and generate MIDI
    let pattern = pattern_data.to_pattern()?;
    let mut composition = Composition::new(pattern_data.tempo_hint.unwrap_or(130));
    composition.add_rhythm_track(pattern.as_ref(), 32);
    composition.save("uk_garage_example.mid")?;
    println!("Generated uk_garage_example.mid");

    // Demonstrate loading patterns from a directory
    let mut library = PatternLibrary::new();

    // First try a non-existent directory to show error handling
    println!("\nTrying to load from non-existent directory:");
    match library.load_from_directory(Path::new("nonexistent_patterns")) {
        Ok(_) => println!("Should not succeed"),
        Err(e) => {
            eprintln!("{:?}", miette::Report::new(e));
        }
    }

    // Then load from the real directory
    library.load_from_directory(Path::new(lobachevsky::library::PATTERN_LIB))?;

    println!("\nAvailable patterns in library:");
    for name in library.list() {
        println!("  - {}", name);
    }

    // Generate MIDI from a library pattern
    if let Some(breakbeat) = library.get("breakbeat") {
        let pattern = breakbeat.to_pattern()?;
        let mut composition = Composition::new(breakbeat.tempo_hint.unwrap_or(140));
        composition.add_rhythm_track(pattern.as_ref(), 32);
        composition.save("breakbeat_example.mid")?;
        println!("\nGenerated breakbeat_example.mid");
    }

    Ok(())
}
