use lobachevsky::Chord;
use lobachevsky::bass::{BassGenerator, BassStrategy};

fn main() {
    let chords = vec![Chord::c_major(), Chord::f_major(), Chord::g_major(), Chord::c_major()];

    println!("Testing different bass strategies:");

    // Root bass
    let root_bass = BassGenerator::new(BassStrategy::Root).with_note_duration(1.0);
    let bass_line = root_bass.generate(&chords, 1);
    println!(
        "Root bass: {:?}",
        bass_line.iter().map(|(n, _, _)| n).collect::<Vec<_>>()
    );

    // Walking bass
    let walking_bass = BassGenerator::new(BassStrategy::Walking).with_note_duration(0.5);
    let bass_line = walking_bass.generate(&chords, 2);
    println!(
        "Walking bass: {:?}",
        bass_line.iter().map(|(n, _, _)| n).collect::<Vec<_>>()
    );

    // Root-fifth pattern
    let root_fifth_bass = BassGenerator::new(BassStrategy::RootFifth).with_note_duration(0.5);
    let bass_line = root_fifth_bass.generate(&chords, 2);
    println!(
        "Root-fifth bass: {:?}",
        bass_line.iter().map(|(n, _, _)| n).collect::<Vec<_>>()
    );
}
