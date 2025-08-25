//! Core musical types and primitives

use std::fmt;

use crate::LobachevskyError;

/// Represents the 12 pitch classes in Western music
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PitchClass {
    #[default]
    C = 0,
    Cs = 1, // C♯ / D♭
    D = 2,
    Ds = 3, // D♯ / E♭
    E = 4,
    F = 5,
    Fs = 6, // F♯ / G♭
    G = 7,
    Gs = 8, // G♯ / A♭
    A = 9,
    As = 10, // A♯ / B♭
    B = 11,
}

impl TryFrom<&str> for PitchClass {
    type Error = crate::LobachevskyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "C" => Ok(PitchClass::C),
            "C#" | "Cs" => Ok(PitchClass::Cs),
            "D" => Ok(PitchClass::D),
            "D#" | "Ds" | "Eb" => Ok(PitchClass::Ds),
            "E" => Ok(PitchClass::E),
            "F" => Ok(PitchClass::F),
            "F#" | "Fs" => Ok(PitchClass::Fs),
            "G" => Ok(PitchClass::G),
            "G#" | "Gs" | "Ab" => Ok(PitchClass::Gs),
            "A" => Ok(PitchClass::A),
            "A#" | "As" | "Bb" => Ok(PitchClass::As),
            "B" => Ok(PitchClass::B),
            _ => Err(LobachevskyError::InvalidPitch {
                input: value.to_string(),
            }),
        }
    }
}

impl PitchClass {
    /// Convert from a semitone value (0-11)
    pub fn from_semitone(semitone: u8) -> Option<Self> {
        match semitone % 12 {
            0 => Some(PitchClass::C),
            1 => Some(PitchClass::Cs),
            2 => Some(PitchClass::D),
            3 => Some(PitchClass::Ds),
            4 => Some(PitchClass::E),
            5 => Some(PitchClass::F),
            6 => Some(PitchClass::Fs),
            7 => Some(PitchClass::G),
            8 => Some(PitchClass::Gs),
            9 => Some(PitchClass::A),
            10 => Some(PitchClass::As),
            11 => Some(PitchClass::B),
            _ => None,
        }
    }

    /// Get the semitone value (0-11)
    pub fn to_semitone(&self) -> u8 {
        *self as u8
    }

    /// Transpose by a number of semitones
    pub fn transpose(&self, semitones: i8) -> Self {
        let current = self.to_semitone() as i8;
        let new_val = ((current + semitones).rem_euclid(12)) as u8;
        Self::from_semitone(new_val).expect("failed to convert semitone {new_val} to PitchClass; exiting")
    }

    /// Get enharmonic equivalent name (for display)
    pub fn enharmonic_name(&self, prefer_sharps: bool) -> &'static str {
        match self {
            PitchClass::C => "C",
            PitchClass::Cs => {
                if prefer_sharps {
                    "C♯"
                } else {
                    "D♭"
                }
            }
            PitchClass::D => "D",
            PitchClass::Ds => {
                if prefer_sharps {
                    "D♯"
                } else {
                    "E♭"
                }
            }
            PitchClass::E => "E",
            PitchClass::F => "F",
            PitchClass::Fs => {
                if prefer_sharps {
                    "F♯"
                } else {
                    "G♭"
                }
            }
            PitchClass::G => "G",
            PitchClass::Gs => {
                if prefer_sharps {
                    "G♯"
                } else {
                    "A♭"
                }
            }
            PitchClass::A => "A",
            PitchClass::As => {
                if prefer_sharps {
                    "A♯"
                } else {
                    "B♭"
                }
            }
            PitchClass::B => "B",
        }
    }
}

impl fmt::Display for PitchClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.enharmonic_name(true))
    }
}

/// Represents a specific note with octave
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Note {
    pub pitch_class: PitchClass,
    pub octave: i8,
}

impl Default for Note {
    fn default() -> Self {
        Self {
            pitch_class: PitchClass::C,
            octave: 3,
        }
    }
}

impl From<&str> for Note {
    fn from(value: &str) -> Self {
        Note {
            pitch_class: PitchClass::try_from(value).unwrap_or_default(),
            ..Default::default()
        }
    }
}

impl Note {
    /// Create a new note
    pub fn new(pitch_class: PitchClass, octave: i8) -> Self {
        Note { pitch_class, octave }
    }

    /// Convert to MIDI note number (0-127)
    pub fn to_midi(&self) -> u8 {
        let base = (self.octave + 1) * 12;
        (base + self.pitch_class.to_semitone() as i8) as u8
    }

    /// Create from MIDI note number
    pub fn from_midi(midi: u8) -> Self {
        let octave = (midi / 12) as i8 - 1;
        let pitch_class =
            PitchClass::from_semitone(midi % 12).expect("failed to convert semitone {new_val} to PitchClass; exiting");
        Note { pitch_class, octave }
    }

    /// Transpose by semitones
    pub fn transpose(&self, semitones: i8) -> Self {
        let new_midi = (self.to_midi() as i8 + semitones) as u8;
        Self::from_midi(new_midi)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.pitch_class, self.octave)
    }
}

/// Musical modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    Ionian, // Major scale (W-W-H-W-W-W-H)
    Dorian,     // Minor with ♮6 (W-H-W-W-W-H-W)
    Phrygian,   // Minor with ♭2 (H-W-W-W-H-W-W)
    Lydian,     // Major with ♯4 (W-W-W-H-W-W-H)
    Mixolydian, // Major with ♭7 (W-W-H-W-W-H-W)
    Aeolian,    // Natural minor (W-H-W-W-H-W-W)
    Locrian,    // Diminished scale (H-W-W-H-W-W-W)
}

impl TryFrom<&str> for Mode {
    type Error = crate::LobachevskyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "ionian" => Ok(Mode::Ionian),
            "dorian" => Ok(Mode::Dorian),
            "phrygian" => Ok(Mode::Phrygian),
            "lydian" => Ok(Mode::Lydian),
            "mixolydian" => Ok(Mode::Mixolydian),
            "aeolian" => Ok(Mode::Aeolian),
            "locrian" => Ok(Mode::Locrian),
            _ => Err(LobachevskyError::InvalidMode {
                input: value.to_string(),
            }),
        }
    }
}

impl Mode {
    /// Get the interval pattern for this mode (in semitones from tonic)
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            Mode::Ionian => vec![0, 2, 4, 5, 7, 9, 11],     // W-W-H-W-W-W-H
            Mode::Dorian => vec![0, 2, 3, 5, 7, 9, 10],     // W-H-W-W-W-H-W
            Mode::Phrygian => vec![0, 1, 3, 5, 7, 8, 10],   // H-W-W-W-H-W-W
            Mode::Lydian => vec![0, 2, 4, 6, 7, 9, 11],     // W-W-W-H-W-W-H
            Mode::Mixolydian => vec![0, 2, 4, 5, 7, 9, 10], // W-W-H-W-W-H-W
            Mode::Aeolian => vec![0, 2, 3, 5, 7, 8, 10],    // W-H-W-W-H-W-W
            Mode::Locrian => vec![0, 1, 3, 5, 6, 8, 10],    // H-W-W-H-W-W-W
        }
    }

    /// Get the chord qualities for each degree of this mode
    /// Returns (major, minor, diminished) pattern for scale degrees I-VII
    pub fn chord_qualities(&self) -> Vec<ChordQuality> {
        match self {
            Mode::Ionian => vec![
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Minor,
                ChordQuality::Major,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Diminished,
            ],
            Mode::Dorian => vec![
                ChordQuality::Minor,
                ChordQuality::Minor,
                ChordQuality::Major,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Diminished,
                ChordQuality::Major,
            ],
            Mode::Phrygian => vec![
                ChordQuality::Minor,
                ChordQuality::Major,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Diminished,
                ChordQuality::Major,
                ChordQuality::Minor,
            ],
            Mode::Lydian => vec![
                ChordQuality::Major,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Diminished,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Minor,
            ],
            Mode::Mixolydian => vec![
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Diminished,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Minor,
                ChordQuality::Major,
            ],
            Mode::Aeolian => vec![
                ChordQuality::Minor,
                ChordQuality::Diminished,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Minor,
                ChordQuality::Major,
                ChordQuality::Major,
            ],
            Mode::Locrian => vec![
                ChordQuality::Diminished,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Minor,
                ChordQuality::Major,
                ChordQuality::Major,
                ChordQuality::Minor,
            ],
        }
    }

    /// Get all valid triads in this mode for a given tonic
    pub fn triads(&self, tonic: PitchClass) -> Vec<Chord> {
        let intervals = self.intervals();
        let qualities = self.chord_qualities();

        intervals
            .iter()
            .zip(qualities.iter())
            .map(|(&interval, &quality)| {
                let root = tonic.transpose(interval as i8);
                Chord::new(root, quality)
            })
            .collect()
    }

    /// Check if a chord belongs to this mode with the given tonic
    pub fn contains_chord(&self, tonic: PitchClass, chord: Chord) -> bool {
        self.triads(tonic).contains(&chord)
    }

    /// Get mode name for display
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Ionian => "Ionian (Major)",
            Mode::Dorian => "Dorian",
            Mode::Phrygian => "Phrygian",
            Mode::Lydian => "Lydian",
            Mode::Mixolydian => "Mixolydian",
            Mode::Aeolian => "Aeolian (Natural Minor)",
            Mode::Locrian => "Locrian",
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Chord quality including triads, seventh chords, and suspended chords
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChordQuality {
    // Basic triads
    #[default]
    Major,
    Minor,
    Diminished,
    Augmented,

    // Seventh chords
    MajorSeventh,          // maj7
    MinorSeventh,          // m7
    DominantSeventh,       // 7
    MinorMajorSeventh,     // mMaj7
    HalfDiminished,        // ø7 (m7♭5)
    FullyDiminished,       // °7
    AugmentedMajorSeventh, // +maj7

    // Suspended chords
    Sus2,      // sus2
    Sus4,      // sus4
    SevenSus2, // 7sus2
    SevenSus4, // 7sus4
}

impl ChordQuality {
    /// Get the intervals from root for this chord quality (in semitones)
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            // Basic triads
            ChordQuality::Major => vec![0, 4, 7], // Root, Major 3rd, Perfect 5th
            ChordQuality::Minor => vec![0, 3, 7], // Root, Minor 3rd, Perfect 5th
            ChordQuality::Diminished => vec![0, 3, 6], // Root, Minor 3rd, Diminished 5th
            ChordQuality::Augmented => vec![0, 4, 8], // Root, Major 3rd, Augmented 5th

            // Seventh chords
            ChordQuality::MajorSeventh => vec![0, 4, 7, 11], // Root, Maj3, P5, Maj7
            ChordQuality::MinorSeventh => vec![0, 3, 7, 10], // Root, Min3, P5, Min7
            ChordQuality::DominantSeventh => vec![0, 4, 7, 10], // Root, Maj3, P5, Min7
            ChordQuality::MinorMajorSeventh => vec![0, 3, 7, 11], // Root, Min3, P5, Maj7
            ChordQuality::HalfDiminished => vec![0, 3, 6, 10], // Root, Min3, Dim5, Min7
            ChordQuality::FullyDiminished => vec![0, 3, 6, 9], // Root, Min3, Dim5, Dim7
            ChordQuality::AugmentedMajorSeventh => vec![0, 4, 8, 11], // Root, Maj3, Aug5, Maj7

            // Suspended chords
            ChordQuality::Sus2 => vec![0, 2, 7],          // Root, Maj2, P5
            ChordQuality::Sus4 => vec![0, 5, 7],          // Root, P4, P5
            ChordQuality::SevenSus2 => vec![0, 2, 7, 10], // Root, Maj2, P5, Min7
            ChordQuality::SevenSus4 => vec![0, 5, 7, 10], // Root, P4, P5, Min7
        }
    }

    /// Symbol for chord notation
    pub fn symbol(&self) -> &'static str {
        match self {
            // Basic triads
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Diminished => "°",
            ChordQuality::Augmented => "+",

            // Seventh chords
            ChordQuality::MajorSeventh => "maj7",
            ChordQuality::MinorSeventh => "m7",
            ChordQuality::DominantSeventh => "7",
            ChordQuality::MinorMajorSeventh => "mMaj7",
            ChordQuality::HalfDiminished => "ø7",
            ChordQuality::FullyDiminished => "°7",
            ChordQuality::AugmentedMajorSeventh => "+maj7",

            // Suspended chords
            ChordQuality::Sus2 => "sus2",
            ChordQuality::Sus4 => "sus4",
            ChordQuality::SevenSus2 => "7sus2",
            ChordQuality::SevenSus4 => "7sus4",
        }
    }

    pub fn split_root_quality(input: &str) -> (&str, Self) {
        // Check suffixes from longest to shortest to avoid partial matches
        for (suffix, quality) in &Self::SYMBOLS[..] {
            if let Some(root_str) = input.strip_suffix(suffix) {
                return (root_str, *quality);
            }
        }
        // Default to Major if no suffix matches
        (input, Self::Major)
    }

    // Ordered from longest to shortest suffix to ensure proper matching
    const SYMBOLS: [(&str, Self); 19] = [
        ("7sus4", ChordQuality::SevenSus4),
        ("7sus2", ChordQuality::SevenSus2),
        ("+maj7", ChordQuality::AugmentedMajorSeventh),
        ("mMaj7", ChordQuality::MinorMajorSeventh),
        ("m7b5", ChordQuality::HalfDiminished),  // Alternative spelling
        ("dim7", ChordQuality::FullyDiminished), // Alternative spelling
        ("maj7", ChordQuality::MajorSeventh),
        ("sus4", ChordQuality::Sus4),
        ("sus2", ChordQuality::Sus2),
        ("aug", ChordQuality::Augmented),  // Alternative spelling
        ("dim", ChordQuality::Diminished), // Alternative spelling
        ("m7", ChordQuality::MinorSeventh),
        ("°7", ChordQuality::FullyDiminished),
        ("ø7", ChordQuality::HalfDiminished),
        ("7", ChordQuality::DominantSeventh),
        ("+", ChordQuality::Augmented),
        ("°", ChordQuality::Diminished),
        ("m", ChordQuality::Minor),
        ("", ChordQuality::Major),
    ];

    /// Check if this is a triad (3 notes)
    pub fn is_triad(&self) -> bool {
        matches!(
            self,
            ChordQuality::Major
                | ChordQuality::Minor
                | ChordQuality::Diminished
                | ChordQuality::Augmented
                | ChordQuality::Sus2
                | ChordQuality::Sus4
        )
    }

    /// Check if this is a seventh chord (4 notes)
    pub fn is_seventh(&self) -> bool {
        matches!(
            self,
            ChordQuality::MajorSeventh
                | ChordQuality::MinorSeventh
                | ChordQuality::DominantSeventh
                | ChordQuality::MinorMajorSeventh
                | ChordQuality::HalfDiminished
                | ChordQuality::FullyDiminished
                | ChordQuality::AugmentedMajorSeventh
                | ChordQuality::SevenSus2
                | ChordQuality::SevenSus4
        )
    }

    /// Check if this is a suspended chord
    pub fn is_suspended(&self) -> bool {
        matches!(
            self,
            ChordQuality::Sus2 | ChordQuality::Sus4 | ChordQuality::SevenSus2 | ChordQuality::SevenSus4
        )
    }

    /// Get the underlying triad for transformation purposes
    /// For seventh chords, returns the triad portion
    /// For sus chords, returns the chord with suspension resolved to major
    pub fn underlying_triad(&self) -> ChordQuality {
        match self {
            // Triads return themselves
            ChordQuality::Major | ChordQuality::Minor | ChordQuality::Diminished | ChordQuality::Augmented => *self,

            // Sus chords resolve to major (this is somewhat arbitrary but common)
            ChordQuality::Sus2 | ChordQuality::Sus4 => ChordQuality::Major,

            // Seventh chords return their triad basis
            ChordQuality::MajorSeventh => ChordQuality::Major,
            ChordQuality::MinorSeventh => ChordQuality::Minor,
            ChordQuality::DominantSeventh => ChordQuality::Major,
            ChordQuality::MinorMajorSeventh => ChordQuality::Minor,
            ChordQuality::HalfDiminished => ChordQuality::Diminished,
            ChordQuality::FullyDiminished => ChordQuality::Diminished,
            ChordQuality::AugmentedMajorSeventh => ChordQuality::Augmented,
            ChordQuality::SevenSus2 | ChordQuality::SevenSus4 => ChordQuality::Major,
        }
    }
}

/// Represents a chord
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Chord {
    pub root: PitchClass,
    pub quality: ChordQuality,
}

impl Chord {
    /// Create a new chord
    pub fn new(root: PitchClass, quality: ChordQuality) -> Self {
        Chord { root, quality }
    }

    /// Create a major chord
    pub fn major(root: PitchClass) -> Self {
        Chord::new(root, ChordQuality::Major)
    }

    /// Create a minor chord
    pub fn minor(root: PitchClass) -> Self {
        Chord::new(root, ChordQuality::Minor)
    }

    /// Create a seventh chord
    pub fn seventh(root: PitchClass, quality: ChordQuality) -> Self {
        debug_assert!(quality.is_seventh(), "Quality must be a seventh chord type");
        Chord::new(root, quality)
    }

    /// Create a suspended chord
    pub fn suspended(root: PitchClass, quality: ChordQuality) -> Self {
        debug_assert!(quality.is_suspended(), "Quality must be a suspended chord type");
        Chord::new(root, quality)
    }

    /// Common constructors for convenience
    pub fn c_major() -> Self {
        Chord::major(PitchClass::C)
    }
    pub fn f_major() -> Self {
        Chord::major(PitchClass::F)
    }
    pub fn g_major() -> Self {
        Chord::major(PitchClass::G)
    }
    pub fn a_minor() -> Self {
        Chord::minor(PitchClass::A)
    }
    pub fn d_minor() -> Self {
        Chord::minor(PitchClass::D)
    }
    pub fn e_minor() -> Self {
        Chord::minor(PitchClass::E)
    }

    // Seventh chord constructors
    pub fn c_major_seventh() -> Self {
        Chord::seventh(PitchClass::C, ChordQuality::MajorSeventh)
    }
    pub fn c_dominant_seventh() -> Self {
        Chord::seventh(PitchClass::C, ChordQuality::DominantSeventh)
    }
    pub fn a_minor_seventh() -> Self {
        Chord::seventh(PitchClass::A, ChordQuality::MinorSeventh)
    }

    // Suspended chord constructors
    pub fn f_sus2() -> Self {
        Chord::suspended(PitchClass::F, ChordQuality::Sus2)
    }
    pub fn f_sus4() -> Self {
        Chord::suspended(PitchClass::F, ChordQuality::Sus4)
    }

    /// Get the notes of this chord at a specific octave
    pub fn notes(&self, octave: i8) -> Vec<Note> {
        self.quality
            .intervals()
            .iter()
            .map(|&interval| {
                let pitch = self.root.transpose(interval as i8);
                let oct_adjust = if self.root.to_semitone() + interval > 11 { 1 } else { 0 };
                Note::new(pitch, octave + oct_adjust)
            })
            .collect()
    }

    /// Get chord name for display
    pub fn name(&self, prefer_sharps: bool) -> String {
        format!("{}{}", self.root.enharmonic_name(prefer_sharps), self.quality.symbol())
    }
}

impl TryFrom<&str> for Chord {
    type Error = LobachevskyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (root_str, quality) = ChordQuality::split_root_quality(value);
        let root = PitchClass::try_from(root_str)?;
        Ok(Chord::new(root, quality))
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_transpose_pitch_classes() {
        assert_eq!(PitchClass::C.transpose(3), PitchClass::Ds);
        assert_eq!(PitchClass::A.transpose(3), PitchClass::C);
        assert_eq!(PitchClass::B.transpose(1), PitchClass::C);
        assert_eq!(PitchClass::C.transpose(-1), PitchClass::B);
    }

    #[test]
    fn converting_notes_to_midi() {
        let middle_c = Note::new(PitchClass::C, 4);
        assert_eq!(middle_c.to_midi(), 60);

        let a440 = Note::new(PitchClass::A, 4);
        assert_eq!(a440.to_midi(), 69);

        assert_eq!(Note::from_midi(60), middle_c);
    }

    #[test]
    fn chord_notes() {
        let c_major = Chord::c_major();
        let notes = c_major.notes(4);
        assert_eq!(notes[0], Note::new(PitchClass::C, 4));
        assert_eq!(notes[1], Note::new(PitchClass::E, 4));
        assert_eq!(notes[2], Note::new(PitchClass::G, 4));
    }

    #[test]
    fn can_parse_chords() {
        // Test all chord quality suffixes from the SYMBOLS constant
        // The SYMBOLS constant has suffixes ordered from longest to shortest for proper
        // parsing

        // Test basic major chord (empty suffix)
        assert_eq!(("C", ChordQuality::Major), ChordQuality::split_root_quality("C"));

        // Test minor chord
        assert_eq!(("Eb", ChordQuality::Minor), ChordQuality::split_root_quality("Ebm"));

        // Test diminished chord
        assert_eq!(
            ("Db", ChordQuality::Diminished),
            ChordQuality::split_root_quality("Db°")
        );

        // Test augmented chord
        assert_eq!(("Ab", ChordQuality::Augmented), ChordQuality::split_root_quality("Ab+"));

        // Test dominant seventh
        assert_eq!(
            ("C#", ChordQuality::DominantSeventh),
            ChordQuality::split_root_quality("C#7")
        );

        // Test half diminished seventh
        assert_eq!(
            ("B", ChordQuality::HalfDiminished),
            ChordQuality::split_root_quality("Bø7")
        );

        // Test fully diminished seventh
        assert_eq!(
            ("E", ChordQuality::FullyDiminished),
            ChordQuality::split_root_quality("E°7")
        );

        // Test minor seventh
        assert_eq!(
            ("A", ChordQuality::MinorSeventh),
            ChordQuality::split_root_quality("Am7")
        );

        // Test major seventh
        assert_eq!(
            ("G", ChordQuality::MajorSeventh),
            ChordQuality::split_root_quality("Gmaj7")
        );

        // Test minor major seventh
        assert_eq!(
            ("D", ChordQuality::MinorMajorSeventh),
            ChordQuality::split_root_quality("DmMaj7")
        );

        // Test augmented major seventh
        assert_eq!(
            ("Bb", ChordQuality::AugmentedMajorSeventh),
            ChordQuality::split_root_quality("Bb+maj7")
        );

        // Test suspended chords
        assert_eq!(("G#", ChordQuality::Sus2), ChordQuality::split_root_quality("G#sus2"));
        assert_eq!(("F", ChordQuality::Sus4), ChordQuality::split_root_quality("Fsus4"));

        // Test seventh suspended chords
        assert_eq!(
            ("F#", ChordQuality::SevenSus2),
            ChordQuality::split_root_quality("F#7sus2")
        );
        assert_eq!(
            ("C", ChordQuality::SevenSus4),
            ChordQuality::split_root_quality("C7sus4")
        );

        // Test alternative spellings
        assert_eq!(
            ("B", ChordQuality::HalfDiminished),
            ChordQuality::split_root_quality("Bm7b5")
        );
        assert_eq!(
            ("E", ChordQuality::FullyDiminished),
            ChordQuality::split_root_quality("Edim7")
        );
        assert_eq!(
            ("D", ChordQuality::Diminished),
            ChordQuality::split_root_quality("Ddim")
        );
        assert_eq!(("C", ChordQuality::Augmented), ChordQuality::split_root_quality("Caug"));

        // Verify alternative spellings match primary spellings
        assert_eq!(
            ChordQuality::split_root_quality("Fø7"),
            ChordQuality::split_root_quality("Fm7b5")
        );
        assert_eq!(
            ChordQuality::split_root_quality("G°7"),
            ChordQuality::split_root_quality("Gdim7")
        );
        assert_eq!(
            ChordQuality::split_root_quality("A°"),
            ChordQuality::split_root_quality("Adim")
        );
        assert_eq!(
            ChordQuality::split_root_quality("D+"),
            ChordQuality::split_root_quality("Daug")
        );

        // Test edge cases with different root note formats
        assert_eq!(("F#", ChordQuality::Major), ChordQuality::split_root_quality("F#"));
        assert_eq!(("Bb", ChordQuality::Minor), ChordQuality::split_root_quality("Bbm"));
        assert_eq!(
            ("D#", ChordQuality::DominantSeventh),
            ChordQuality::split_root_quality("D#7")
        );
    }
}
