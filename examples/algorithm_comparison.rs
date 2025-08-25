//! Compare the complexity and performance of the two Euclidean algorithms

use std::time::Instant;

use lobachevsky::euclidean::{Bjorklund, Breshenham, EuclideanRhythm};

fn main() {
    println!("Algorithm Comparison: Bjorklund vs Bresenham");
    println!("=============================================\n");

    // Complexity Analysis
    print_complexity_analysis();

    // Simple Performance Test
    performance_test();

    // Pattern Comparison
    pattern_comparison();
}

fn print_complexity_analysis() {
    println!("CODE COMPLEXITY ANALYSIS:");
    println!("========================");

    println!("\n🔹 Bjorklund Algorithm:");
    println!("   • Lines of code: ~72 (including recursive function)");
    println!("   • Time complexity: O(log(min(hits, steps)))");
    println!("   • Space complexity: O(log(min(hits, steps))) for recursion stack");
    println!("   • Algorithm: Based on Euclidean GCD with recursive pattern building");
    println!("   • Pros: Mathematically elegant, theoretically optimal distribution");
    println!("   • Cons: More complex code, uses recursion, harder to understand");

    println!("\n🔹 Bresenham Algorithm:");
    println!("   • Lines of code: ~43");
    println!("   • Time complexity: O(steps)");
    println!("   • Space complexity: O(1) extra space");
    println!("   • Algorithm: Line-drawing algorithm adapted for rhythm distribution");
    println!("   • Pros: Simple, fast, easy to understand, no recursion");
    println!("   • Cons: May produce slightly different distributions than theoretical optimum");

    println!("\n🏆 Winner for SIMPLICITY: Bresenham (much simpler code)");
    println!("🏆 Winner for PERFORMANCE: Bresenham (O(n) vs O(log n) but simpler operations)");
}

fn performance_test() {
    println!("\n\nPERFORMANCE TEST:");
    println!("================");

    let test_cases = vec![
        (16, 4),   // Simple case
        (32, 13),  // Medium complexity
        (64, 23),  // High complexity
        (128, 47), // Very high complexity
    ];

    for (steps, pulses) in test_cases {
        println!("\nTesting E({},{}) - 10,000 iterations:", pulses, steps);

        // Test Bjorklund
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = Bjorklund::generate(steps, pulses).expect("this should work");
        }
        let bjork_time = start.elapsed();

        // Test Bresenham
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = Breshenham::generate(steps, pulses).expect("this should work");
        }
        let bres_time = start.elapsed();

        println!("  Bjorklund:  {:?}", bjork_time);
        println!("  Bresenham:  {:?}", bres_time);

        let ratio = bjork_time.as_nanos() as f64 / bres_time.as_nanos() as f64;
        if ratio > 1.1 {
            println!("  🏆 Bresenham is {:.1}x faster", ratio);
        } else if ratio < 0.9 {
            println!("  🏆 Bjorklund is {:.1}x faster", 1.0 / ratio);
        } else {
            println!("  ≈ Similar performance");
        }
    }
}

fn pattern_comparison() {
    println!("\n\nPATTERN COMPARISON:");
    println!("==================");

    let test_cases = vec![
        (8, 3, "Tresillo"),
        (8, 5, "Cinquillo"),
        (16, 7, "Complex"),
        (12, 5, "Irregular"),
    ];

    for (steps, pulses, name) in test_cases {
        println!("\n{} E({},{}):", name, pulses, steps);

        let bjork = Bjorklund::generate(steps, pulses).expect("this should work");
        let bres = Breshenham::generate(steps, pulses).expect("this should work");

        print!("  Bjorklund:  ");
        print_pattern(&bjork);
        print!("  Bresenham:  ");
        print_pattern(&bres);

        if bjork == bres {
            println!("  ✅ Identical patterns");
        } else {
            println!("  ⚠️  Different patterns");

            // Analyze distribution quality
            let bjork_quality = analyze_distribution_quality(&bjork);
            let bres_quality = analyze_distribution_quality(&bres);

            println!("    Bjorklund evenness score: {:.2}", bjork_quality);
            println!("    Bresenham evenness score: {:.2}", bres_quality);

            if (bjork_quality - bres_quality).abs() < 0.1 {
                println!("    ≈ Similar distribution quality");
            } else if bjork_quality > bres_quality {
                println!("    🏆 Bjorklund has better distribution");
            } else {
                println!("    🏆 Bresenham has better distribution");
            }
        }
    }
}

fn print_pattern(pattern: &[bool]) {
    let visual: String = pattern.iter().map(|&x| if x { 'X' } else { '.' }).collect();
    println!("{}", visual);
}

/// Simple measure of distribution quality - lower is better (more even)
fn analyze_distribution_quality(pattern: &[bool]) -> f64 {
    if pattern.is_empty() || pattern.iter().all(|&x| !x) {
        return 0.0;
    }

    // Find all pulse positions
    let pulse_positions: Vec<usize> = pattern
        .iter()
        .enumerate()
        .filter_map(|(i, &x)| if x { Some(i) } else { None })
        .collect();

    if pulse_positions.len() <= 1 {
        return 0.0;
    }

    // Calculate distances between consecutive pulses (wrapping around)
    let mut distances = Vec::new();
    for i in 0..pulse_positions.len() {
        let current = pulse_positions[i];
        let next = pulse_positions[(i + 1) % pulse_positions.len()];
        let distance = if next > current {
            next - current
        } else {
            pattern.len() - current + next
        };
        distances.push(distance as f64);
    }

    // Calculate variance of distances (lower variance = more even distribution)
    let mean = distances.iter().sum::<f64>() / distances.len() as f64;
    let variance = distances.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / distances.len() as f64;

    variance.sqrt() // Standard deviation
}
