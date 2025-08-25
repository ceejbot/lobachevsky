//! Test the new isochronic brainwave entrainment patterns

use lobachevsky::rhythm::PatternLibrary;

fn main() {
    println!("🧠 Testing Isochronic Brainwave Entrainment Patterns");
    println!("===================================================");

    // Create pattern library
    let mut library = PatternLibrary::new();

    // Load patterns from directory
    match library.load_from_directory(std::path::Path::new("patterns")) {
        Ok(_) => println!("✅ Successfully loaded patterns from directory"),
        Err(e) => {
            println!("❌ Failed to load patterns: {:?}", e);
            return;
        }
    }

    // Test isochronic and enhanced patterns
    let isochronic_patterns = [
        ("deep_hypnotic_alpha", "Alpha-enhanced hypnotic pattern"),
        ("coding_flow_theta", "Theta-enhanced coding flow"),
        ("focus_minimal_gamma", "Gamma-enhanced minimal focus"),
        ("pure_alpha_entrainment", "Pure alpha wave entrainment"),
        ("theta_deep_work", "Theta waves for deep creative work"),
        ("gamma_peak_focus", "Gamma waves for peak performance"),
        ("beta_active_concentration", "Beta waves for analytical thinking"),
        ("mixed_entrainment_flow", "Multi-frequency flow state"),
    ];

    println!("\n🎵 BRAINWAVE ENTRAINMENT PATTERNS:");
    println!("==================================");

    for (pattern_name, _description) in &isochronic_patterns {
        println!("\n--- Testing {} ---", pattern_name);

        if let Some(pattern_data) = library.get(pattern_name) {
            println!("✅ Found pattern: {}", pattern_data.name);
            if let Some(desc) = &pattern_data.description {
                println!("   Description: {}", desc);
            }
            if let Some(tempo) = pattern_data.tempo_hint {
                println!("   Suggested tempo: {} BPM", tempo);
            }
            println!("   Layers: {}", pattern_data.layers.len());

            // Test pattern conversion
            match pattern_data.to_pattern() {
                Ok(rhythm_pattern) => {
                    println!("✅ Successfully converted to RhythmPattern");

                    // Test generating events for multiple bars
                    let events_bar0 = rhythm_pattern.events_for_bar(0);
                    let events_bar1 = rhythm_pattern.events_for_bar(1);

                    println!("   🎼 Generated {} events for bar 0", events_bar0.len());
                    println!("   🎼 Generated {} events for bar 1", events_bar1.len());
                    println!("   📊 Pattern length: {} bars", rhythm_pattern.pattern_length());

                    // Show first few events for inspection
                    if !events_bar0.is_empty() {
                        println!("   🔍 First 3 events:");
                        for (i, event) in events_bar0.iter().take(3).enumerate() {
                            println!(
                                "      {}. {:?} at beat {:.3} (vel: {})",
                                i + 1,
                                event.voice,
                                event.beat.0,
                                event.velocity
                            );
                        }
                    }
                }
                Err(e) => println!("❌ Failed to convert pattern: {:?}", e),
            }
        } else {
            println!("❌ Pattern not found: {}", pattern_name);
        }
    }

    println!("\n🧠 BRAINWAVE FREQUENCY GUIDE:");
    println!("=============================");
    println!("• Delta (0.5-4 Hz): Deep sleep, healing, regeneration");
    println!("• Theta (4-8 Hz): Deep meditation, creativity, REM sleep");
    println!("• Alpha (8-13 Hz): Relaxed focus, light meditation, flow states");
    println!("• Beta (13-30 Hz): Alert concentration, active thinking");
    println!("• Gamma (30+ Hz): High-level cognitive processing, peak awareness");

    println!("\n🎯 USAGE RECOMMENDATIONS:");
    println!("=========================");
    println!("• Alpha patterns: Perfect for programming and focused work");
    println!("• Theta patterns: Ideal for creative problem-solving and insight");
    println!("• Beta patterns: Best for analytical tasks and active concentration");
    println!("• Gamma patterns: Use for peak cognitive performance and complex tasks");
    println!("• Mixed patterns: Great for extended flow state sessions");

    println!("\n🎧 Isochronic pattern testing complete! 🎧");
}
