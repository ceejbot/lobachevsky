use lobachevsky::Chord;
use lobachevsky::call_response::{CallResponseGenerator, CallResponseType};

fn main() {
    let chords = vec![Chord::c_major(), Chord::f_major(), Chord::g_major(), Chord::c_major()];

    println!("Testing call-and-response patterns:");

    // Echo pattern
    let echo_gen = CallResponseGenerator::new(CallResponseType::Echo, 3)
        .with_note_duration(0.5)
        .with_gap(0.25);
    let echo_phrases = echo_gen.generate(&chords);
    println!("Echo: {} phrases", echo_phrases.len());
    for (i, phrase) in echo_phrases.iter().enumerate() {
        println!(
            "  Phrase {}: {} total notes, duration: {:.2}",
            i + 1,
            phrase.all_notes().len(),
            phrase.total_duration()
        );
    }

    // Sequence (perfect fifth)
    let sequence_gen = CallResponseGenerator::new(CallResponseType::Sequence(7), 2).with_note_duration(0.75);
    let sequence_phrases = sequence_gen.generate(&chords);
    println!("Sequence (+7): {} phrases", sequence_phrases.len());

    // Inversion
    let inversion_gen = CallResponseGenerator::new(CallResponseType::Inversion, 4)
        .with_note_duration(0.25)
        .with_octave_range(4, 6);
    let inversion_phrases = inversion_gen.generate(&chords);
    println!("Inversion: {} phrases", inversion_phrases.len());

    // Complement
    let complement_gen = CallResponseGenerator::new(CallResponseType::Complement, 3);
    let complement_phrases = complement_gen.generate(&chords);
    println!("Complement: {} phrases", complement_phrases.len());
}
