use lobachevsky::adaptive::{AdaptiveHarmonyPattern, AdaptiveRhythmPattern};
use lobachevsky::analysis::{EnergyLevel, HarmonicAnalyzer};
use lobachevsky::midi::Composition;
use lobachevsky::morphing::{HarmonyInterpolationMethod, MorphingHarmonyPattern, MorphingRhythmPattern};
use lobachevsky::rhythm::{DrumVoice, EuclideanPattern, RhythmPattern};
use lobachevsky::{Chord, PitchClass, Transform};

fn main() {
    println!("🎵 Adaptive and Morphing Pattern Demo");
    println!("=====================================\n");

    // Create test chord progressions with different energy levels
    let low_energy_chords = create_low_energy_progression();
    let high_energy_chords = create_high_energy_progression();

    // Demonstrate harmonic energy analysis
    demonstrate_energy_analysis(&low_energy_chords, &high_energy_chords);

    // Demonstrate adaptive rhythm patterns
    demonstrate_adaptive_rhythm(&low_energy_chords, &high_energy_chords);

    // Demonstrate adaptive harmony patterns
    demonstrate_adaptive_harmony();

    // Demonstrate pattern morphing
    demonstrate_morphing_patterns();

    // Generate a complete composition using adaptive features
    generate_adaptive_composition();

    println!("\n✅ Demo completed! Check the generated MIDI files.");
}

/// Create a low-energy chord progression (smooth voice leading)
fn create_low_energy_progression() -> Vec<Chord> {
    vec![Chord::c_major(), Chord::f_major(), Chord::g_major(), Chord::c_major()]
}

/// Create a high-energy chord progression (larger jumps, complex chords)
fn create_high_energy_progression() -> Vec<Chord> {
    vec![
        Chord::c_major(),
        Chord::new(PitchClass::Fs, lobachevsky::ChordQuality::Major),
        Chord::new(PitchClass::As, lobachevsky::ChordQuality::Diminished),
        Chord::new(PitchClass::E, lobachevsky::ChordQuality::Augmented),
    ]
}

/// Demonstrate harmonic energy analysis
fn demonstrate_energy_analysis(low_energy: &[Chord], high_energy: &[Chord]) {
    println!("🔍 Harmonic Energy Analysis");
    println!("---------------------------");

    let analyzer = HarmonicAnalyzer::new().with_tonic(PitchClass::C);

    let low_energy_value = analyzer.analyze_progression_energy(low_energy);
    let high_energy_value = analyzer.analyze_progression_energy(high_energy);

    println!("Low energy progression (C-F-G-C): {:.3}", low_energy_value);
    println!("  Energy level: {:?}", EnergyLevel::from(low_energy_value));
    println!(
        "  Chords: {:?}",
        low_energy.iter().map(|c| format!("{}", c)).collect::<Vec<_>>()
    );

    println!("\nHigh energy progression: {:.3}", high_energy_value);
    println!("  Energy level: {:?}", EnergyLevel::from(high_energy_value));
    println!(
        "  Chords: {:?}",
        high_energy.iter().map(|c| format!("{}", c)).collect::<Vec<_>>()
    );

    // Analyze individual chord energy
    println!("\nIndividual chord analysis:");
    for (i, chord) in high_energy.iter().enumerate() {
        let prev_chord = if i > 0 { Some(high_energy[i - 1]) } else { None };
        let energy = analyzer.analyze_chord_energy(*chord, prev_chord);
        println!("  {}: {:.3} energy", chord, energy);
    }
    println!();
}

/// Demonstrate adaptive rhythm patterns
fn demonstrate_adaptive_rhythm(low_energy: &[Chord], high_energy: &[Chord]) {
    println!("🥁 Adaptive Rhythm Patterns");
    println!("---------------------------");

    let mut adaptive_pattern = AdaptiveRhythmPattern::new("demo_pattern".to_string(), Some(PitchClass::C));

    // Test with low energy progression
    println!("Low energy rhythm response:");
    adaptive_pattern.update_from_chords(low_energy);
    let low_energy_events = adaptive_pattern.events_for_bar(0);
    println!("  Generated {} drum events", low_energy_events.len());
    println!(
        "  Average velocity: {:.1}",
        low_energy_events.iter().map(|e| e.velocity as f64).sum::<f64>() / low_energy_events.len() as f64
    );

    // Test with high energy progression
    println!("\nHigh energy rhythm response:");
    adaptive_pattern.update_from_chords(high_energy);
    let high_energy_events = adaptive_pattern.events_for_bar(0);
    println!("  Generated {} drum events", high_energy_events.len());
    println!(
        "  Average velocity: {:.1}",
        high_energy_events.iter().map(|e| e.velocity as f64).sum::<f64>() / high_energy_events.len() as f64
    );

    println!("  Pattern should have higher density and velocity for high-energy chords\n");
}

/// Demonstrate adaptive harmony patterns
fn demonstrate_adaptive_harmony() {
    println!("🎹 Adaptive Harmony Patterns");
    println!("----------------------------");

    let base_transforms = vec![Transform::P, Transform::L];
    let mut adaptive_harmony = AdaptiveHarmonyPattern::new(base_transforms, PitchClass::C).with_evolution_factor(0.3);

    println!("Simulating 16-bar evolution:");
    let mut transformations = Vec::new();
    let current_chord = Chord::c_major();

    for bar in 0..16 {
        if let Some(transform) = adaptive_harmony.evolve_pattern(current_chord, bar) {
            transformations.push((bar, transform.clone()));
            println!("  Bar {}: Apply {:?}", bar + 1, transform);
        }
    }

    println!("Generated {} transformations over 16 bars", transformations.len());
    println!("Pattern evolves over time, occasionally trying new transformations\n");
}

/// Demonstrate pattern morphing
fn demonstrate_morphing_patterns() {
    println!("🌊 Pattern Morphing");
    println!("-------------------");

    // Create two different rhythm patterns
    let pattern_a = Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 16));
    let pattern_b = Box::new(EuclideanPattern::new(DrumVoice::Kick, 7, 16));

    let morphing_pattern = MorphingRhythmPattern::new(pattern_a, pattern_b, 8);

    println!("Morphing between two Euclidean patterns over 16 bars:");
    for bar in 0..16 {
        let events = morphing_pattern.events_for_bar(bar);
        let morph_progress = if bar < 8 {
            bar as f64 / 8.0
        } else {
            2.0 - (bar as f64 / 8.0)
        };
        println!(
            "  Bar {}: {} events (morph: {:.2})",
            bar + 1,
            events.len(),
            morph_progress
        );
    }

    // Demonstrate harmony morphing
    println!("\nHarmony pattern morphing:");
    let transforms_a = vec![Transform::P, Transform::L];
    let transforms_b = vec![Transform::R, Transform::L, Transform::P];

    let mut harmony_morph =
        MorphingHarmonyPattern::new(transforms_a, transforms_b, HarmonyInterpolationMethod::Probabilistic);

    for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
        harmony_morph.set_morph_progress(progress);
        if let Some(transform) = harmony_morph.get_transform(0, Chord::c_major()) {
            println!("  Progress {:.2}: {:?}", progress, transform);
        }
    }
    println!();
}

/// Generate a complete adaptive composition
fn generate_adaptive_composition() {
    println!("🎼 Generating Adaptive Composition");
    println!("----------------------------------");

    // Create a progression that changes energy over time
    let progression = create_dynamic_progression();

    // Create adaptive rhythm that responds to this progression
    let mut adaptive_rhythm = AdaptiveRhythmPattern::new("dynamic_composition".to_string(), Some(PitchClass::C));

    // Create morphing patterns for variety
    let pattern_a = Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 16));
    let pattern_b = Box::new(EuclideanPattern::new(DrumVoice::Kick, 6, 16));
    let morphing_rhythm = MorphingRhythmPattern::new(pattern_a, pattern_b, 4);

    // Generate composition
    let mut composition = Composition::new(120);

    // Add harmony track
    composition.add_harmony_track(&progression, 3, 1);

    // Generate adaptive rhythm events
    let mut all_rhythm_events = Vec::new();
    for (bar, chunk) in progression.chunks(4).enumerate() {
        adaptive_rhythm.update_from_chords(chunk);
        let events = adaptive_rhythm.events_for_bar(bar);

        for mut event in events {
            // Adjust timing for bar position
            event.beat.0 += bar as f64 * 4.0;
            all_rhythm_events.push(event);
        }
    }

    // Add morphing rhythm events
    let mut morphing_events = Vec::new();
    for bar in 0..(progression.len().min(16)) {
        let events = morphing_rhythm.events_for_bar(bar);

        for mut event in events {
            // Adjust timing and use different voice
            event.beat.0 += bar as f64 * 4.0;
            event.voice = DrumVoice::HiHatClosed;
            event.velocity = (event.velocity as f64 * 0.7) as u8; // Softer
            morphing_events.push(event);
        }
    }

    // Combine rhythm events
    all_rhythm_events.extend(morphing_events);
    composition.add_rhythm_track_from_events(&all_rhythm_events);

    // Save composition
    composition.save("adaptive_demo.mid").expect("Failed to save MIDI");
    println!("Saved adaptive composition to 'adaptive_demo.mid'");
    println!("  {} chords over {} bars", progression.len(), progression.len());
    println!("  {} rhythm events total", all_rhythm_events.len());
    println!("  Patterns adapt to harmonic energy and morph over time");
}

/// Create a progression with varying energy levels
fn create_dynamic_progression() -> Vec<Chord> {
    vec![
        // Low energy start (smooth)
        Chord::c_major(),
        Chord::f_major(),
        Chord::g_major(),
        Chord::c_major(),
        // Building energy (larger jumps)
        Chord::new(PitchClass::A, lobachevsky::ChordQuality::Minor),
        Chord::new(PitchClass::E, lobachevsky::ChordQuality::Major),
        Chord::new(PitchClass::Fs, lobachevsky::ChordQuality::Minor),
        Chord::new(PitchClass::B, lobachevsky::ChordQuality::Major),
        // High energy climax (complex chords, tritones)
        Chord::new(PitchClass::C, lobachevsky::ChordQuality::Major),
        Chord::new(PitchClass::Fs, lobachevsky::ChordQuality::Diminished),
        Chord::new(PitchClass::As, lobachevsky::ChordQuality::Augmented),
        Chord::new(PitchClass::E, lobachevsky::ChordQuality::Minor),
        // Resolution back to low energy
        Chord::f_major(),
        Chord::g_major(),
        Chord::c_major(),
        Chord::c_major(),
    ]
}
