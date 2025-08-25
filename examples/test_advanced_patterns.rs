//! Test the new advanced rhythm patterns with TOML loading

use lobachevsky::rhythm::PatternLibrary;

fn main() {
    println!("Testing Advanced Rhythm Patterns");
    println!("=================================");

    // Create pattern library
    let mut library = PatternLibrary::new();

    // Load patterns from directory
    match library.load_from_directory(std::path::Path::new("patterns")) {
        Ok(_) => println!("✓ Successfully loaded patterns from directory"),
        Err(e) => {
            println!("✗ Failed to load patterns: {:?}", e);
            return;
        }
    }

    // List available patterns
    let patterns = library.list();
    println!("\nAvailable patterns:");
    for pattern_name in &patterns {
        println!("  - {}", pattern_name);
    }

    // Test loading and converting specific patterns
    let test_patterns = [
        "deep_hypnotic", "coding_flow", "focus_minimal", "ambient_progressive", "trance_builder",
    ];

    for pattern_name in &test_patterns {
        println!("\n--- Testing {} ---", pattern_name);

        if let Some(pattern_data) = library.get(pattern_name) {
            println!("✓ Found pattern: {}", pattern_data.name);
            if let Some(desc) = &pattern_data.description {
                println!("  Description: {}", desc);
            }
            if let Some(tempo) = pattern_data.tempo_hint {
                println!("  Suggested tempo: {} BPM", tempo);
            }
            println!("  Layers: {}", pattern_data.layers.len());

            // Test pattern conversion
            match pattern_data.to_pattern() {
                Ok(_rhythm_pattern) => {
                    println!("✓ Successfully converted to RhythmPattern");
                    // Could test generating events here if needed
                    // let events = rhythm_pattern.events_for_bar(0);
                    // println!("  Generated {} events for bar 0",
                    // events.len());
                }
                Err(e) => println!("✗ Failed to convert pattern: {:?}", e),
            }
        } else {
            println!("✗ Pattern not found: {}", pattern_name);
        }
    }

    println!("\nAdvanced pattern testing complete!");
}
