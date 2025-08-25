//! Compare the two Euclidean rhythm algorithms: Bjorklund vs Bresenham

use lobachevsky::euclidean::{Bjorklund, Breshenham, EuclideanRhythm};

fn main() {
    println!("Comparing Euclidean Rhythm Algorithms");
    println!("=====================================");

    let test_cases = vec![
        (8, 3, "Tresillo (Cuban)"),
        (8, 5, "Cinquillo (Cuban)"),
        (16, 4, "Standard 4/4 kick"),
        (12, 7, "West African bell"),
        (16, 9, "Complex polyrhythm"),
        (8, 2, "Simple kick-snare"),
    ];

    for (steps, pulses, description) in test_cases {
        println!("\n{} - E({},{}):", description, pulses, steps);

        // Test Bjorklund algorithm
        match Bjorklund::generate(steps, pulses) {
            Ok(pattern) => {
                print!("  Bjorklund:  ");
                print_pattern(&pattern);
            }
            Err(e) => {
                println!("  Bjorklund:  Error: {:?}", e);
            }
        }

        // Test Bresenham algorithm
        match Breshenham::generate(steps, pulses) {
            Ok(pattern) => {
                print!("  Bresenham:  ");
                print_pattern(&pattern);
            }
            Err(e) => {
                println!("  Bresenham:  Error: {:?}", e);
            }
        }
    }

    println!("\n\nEdge Cases:");
    println!("===========");

    test_edge_case(8, 0, "No pulses");
    test_edge_case(4, 4, "All pulses");
    test_edge_case(1, 1, "Single step, single pulse");
    test_edge_case(1, 0, "Single step, no pulse");
}

fn test_edge_case(steps: usize, pulses: usize, description: &str) {
    println!("\n{} - E({},{}):", description, pulses, steps);

    let bjork = Bjorklund::generate(steps, pulses).expect("this should work");
    let bres = Breshenham::generate(steps, pulses).expect("this should work");

    print!("  Bjorklund:  ");
    print_pattern(&bjork);
    print!("  Bresenham:  ");
    print_pattern(&bres);

    if bjork == bres {
        println!("  ✓ Algorithms produce identical results");
    } else {
        println!("  ⚠ Algorithms produce different results");
    }
}

fn print_pattern(pattern: &[bool]) {
    // Print as X and . for readability
    let visual: String = pattern.iter().map(|&x| if x { 'X' } else { '.' }).collect();

    println!("{}", visual);
}
