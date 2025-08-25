//! Neo-Riemannian theory and transformations

use std::collections::{HashMap, HashSet};

use crate::LobachevskyError;
use crate::core::{Chord, ChordQuality, Mode, Note, PitchClass};

/// Neo-Riemannian transformations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transform {
    /// Parallel: Changes mode (major ↔ minor) by altering the third
    P,
    /// Relative: Major to relative minor or vice versa
    R,
    /// Leading-tone: Exchange root and fifth, alter to change mode
    L,
    /// Compound transformation (sequence of basic transforms)
    Compound(Vec<Transform>),
}

impl TryFrom<&str> for Transform {
    type Error = LobachevskyError;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        match v.trim().to_ascii_lowercase().as_str() {
            "p" | "parallel" => Ok(Self::P),
            "r" | "relative" => Ok(Self::R),
            "l" | "leading-tone" | "leading" | "leadingtone" | "leading tone" => Ok(Self::L),
            "rp" => Ok(Self::rp()),
            "pr" => Ok(Self::pr()),
            "pl" => Ok(Self::pl()),
            "lp" => Ok(Self::lp()),
            "rl" => Ok(Self::rl()),
            "lr" => Ok(Self::lr()),
            _ => Err(LobachevskyError::InvalidTransformation { input: v.to_string() }),
        }
    }
}

impl Transform {
    /// Common compound transformations
    pub fn rp() -> Self {
        Transform::Compound(vec![Transform::R, Transform::P])
    }
    pub fn pr() -> Self {
        Transform::Compound(vec![Transform::P, Transform::R])
    }
    pub fn pl() -> Self {
        Transform::Compound(vec![Transform::P, Transform::L])
    }
    pub fn lp() -> Self {
        Transform::Compound(vec![Transform::L, Transform::P])
    }
    pub fn rl() -> Self {
        Transform::Compound(vec![Transform::R, Transform::L])
    }
    pub fn lr() -> Self {
        Transform::Compound(vec![Transform::L, Transform::R])
    }
}

/// P3,0 transformations for seventh chords
/// Three voices move by half-step, one stays as common tone
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum P3Transform {
    /// Parallel motion up, with specified common tone
    ParallelUp(CommonTone),
    /// Parallel motion down, with specified common tone
    ParallelDown(CommonTone),
    /// Contrary motion, with specified common tone
    Contrary(CommonTone),
}

/// Which chord tone remains common in P3,0 transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonTone {
    Root,    // 1st
    Third,   // 3rd
    Fifth,   // 5th
    Seventh, // 7th
}

/// Trait for chords that can be transformed using neo-Riemannian operations
pub trait Transformable: Clone + Copy + PartialEq + Eq {
    /// Apply a basic neo-Riemannian transformation (P, R, L)
    fn transform_basic(&self, transform: &Transform) -> Option<Self>;

    /// Apply a P3,0 transformation (for seventh chords)
    fn transform_p3(&self, _transform: &P3Transform) -> Option<Self> {
        None // Default implementation - only seventh chords support this
    }

    /// Get the notes of this chord at a specific octave
    fn notes(&self, octave: i8) -> Vec<Note>;

    /// Get the underlying triad for basic transformations
    fn underlying_triad(&self) -> Option<Chord>;

    /// Check if this can be transformed with basic P/R/L operations
    fn supports_basic_transforms(&self) -> bool {
        self.underlying_triad().is_some()
    }

    /// Check if this supports P3,0 transformations
    fn supports_p3_transforms(&self) -> bool {
        false // Default implementation - only seventh chords support this
    }
}

/// Neo-Riemannian transformation engine
pub struct NeoRiemannian {
    transformations: HashMap<Chord, TransformMap>,
}

struct TransformMap {
    p: Chord,
    r: Chord,
    l: Chord,
}

impl NeoRiemannian {
    /// Create a new neo-Riemannian transformer with all mappings
    pub fn new() -> Self {
        let mut transformations = HashMap::new();

        // Major chord transformations
        use PitchClass::*;

        // C major and related
        transformations.insert(
            Chord::major(C),
            TransformMap {
                p: Chord::minor(C),
                r: Chord::minor(A),
                l: Chord::minor(E),
            },
        );

        // C# / Db major
        transformations.insert(
            Chord::major(Cs),
            TransformMap {
                p: Chord::minor(Cs),
                r: Chord::minor(As),
                l: Chord::minor(F),
            },
        );

        // D major
        transformations.insert(
            Chord::major(D),
            TransformMap {
                p: Chord::minor(D),
                r: Chord::minor(B),
                l: Chord::minor(Fs),
            },
        );

        // D# / Eb major
        transformations.insert(
            Chord::major(Ds),
            TransformMap {
                p: Chord::minor(Ds),
                r: Chord::minor(C),
                l: Chord::minor(G),
            },
        );

        // E major
        transformations.insert(
            Chord::major(E),
            TransformMap {
                p: Chord::minor(E),
                r: Chord::minor(Cs),
                l: Chord::minor(Gs),
            },
        );

        // F major
        transformations.insert(
            Chord::major(F),
            TransformMap {
                p: Chord::minor(F),
                r: Chord::minor(D),
                l: Chord::minor(A),
            },
        );

        // F# / Gb major
        transformations.insert(
            Chord::major(Fs),
            TransformMap {
                p: Chord::minor(Fs),
                r: Chord::minor(Ds),
                l: Chord::minor(As),
            },
        );

        // G major
        transformations.insert(
            Chord::major(G),
            TransformMap {
                p: Chord::minor(G),
                r: Chord::minor(E),
                l: Chord::minor(B),
            },
        );

        // G# / Ab major
        transformations.insert(
            Chord::major(Gs),
            TransformMap {
                p: Chord::minor(Gs),
                r: Chord::minor(F),
                l: Chord::minor(C),
            },
        );

        // A major
        transformations.insert(
            Chord::major(A),
            TransformMap {
                p: Chord::minor(A),
                r: Chord::minor(Fs),
                l: Chord::minor(Cs),
            },
        );

        // A# / Bb major
        transformations.insert(
            Chord::major(As),
            TransformMap {
                p: Chord::minor(As),
                r: Chord::minor(G),
                l: Chord::minor(D),
            },
        );

        // B major
        transformations.insert(
            Chord::major(B),
            TransformMap {
                p: Chord::minor(B),
                r: Chord::minor(Gs),
                l: Chord::minor(Ds),
            },
        );

        // Minor chord transformations
        transformations.insert(
            Chord::minor(C),
            TransformMap {
                p: Chord::major(C),
                r: Chord::major(Ds), // Eb
                l: Chord::major(Gs), // Ab
            },
        );

        transformations.insert(
            Chord::minor(Cs),
            TransformMap {
                p: Chord::major(Cs),
                r: Chord::major(E),
                l: Chord::major(A),
            },
        );

        transformations.insert(
            Chord::minor(D),
            TransformMap {
                p: Chord::major(D),
                r: Chord::major(F),
                l: Chord::major(As), // Bb
            },
        );

        transformations.insert(
            Chord::minor(Ds),
            TransformMap {
                p: Chord::major(Ds),
                r: Chord::major(Fs), // F#/Gb
                l: Chord::major(B),
            },
        );

        transformations.insert(
            Chord::minor(E),
            TransformMap {
                p: Chord::major(E),
                r: Chord::major(G),
                l: Chord::major(C),
            },
        );

        transformations.insert(
            Chord::minor(F),
            TransformMap {
                p: Chord::major(F),
                r: Chord::major(Gs), // Ab
                l: Chord::major(Cs), // Db
            },
        );

        transformations.insert(
            Chord::minor(Fs),
            TransformMap {
                p: Chord::major(Fs),
                r: Chord::major(A),
                l: Chord::major(D),
            },
        );

        transformations.insert(
            Chord::minor(G),
            TransformMap {
                p: Chord::major(G),
                r: Chord::major(As), // Bb
                l: Chord::major(Ds), // Eb
            },
        );

        transformations.insert(
            Chord::minor(Gs),
            TransformMap {
                p: Chord::major(Gs),
                r: Chord::major(B),
                l: Chord::major(E),
            },
        );

        transformations.insert(
            Chord::minor(A),
            TransformMap {
                p: Chord::major(A),
                r: Chord::major(C),
                l: Chord::major(F),
            },
        );

        transformations.insert(
            Chord::minor(As),
            TransformMap {
                p: Chord::major(As),
                r: Chord::major(Cs), // C#/Db
                l: Chord::major(Fs), // F#/Gb
            },
        );

        transformations.insert(
            Chord::minor(B),
            TransformMap {
                p: Chord::major(B),
                r: Chord::major(D),
                l: Chord::major(G),
            },
        );

        NeoRiemannian { transformations }
    }

    /// Apply a transformation to a chord
    pub fn transform(&self, chord: Chord, transform: &Transform) -> Option<Chord> {
        match transform {
            Transform::P => self.transformations.get(&chord).map(|t| t.p),
            Transform::R => self.transformations.get(&chord).map(|t| t.r),
            Transform::L => self.transformations.get(&chord).map(|t| t.l),
            Transform::Compound(transforms) => {
                let mut current = chord;
                for t in transforms {
                    current = self.transform(current, t)?;
                }
                Some(current)
            }
        }
    }

    /// Apply a sequence of transformations
    pub fn apply_sequence(&self, start: Chord, transforms: &[Transform]) -> Vec<Chord> {
        let mut result = vec![start];
        let mut current = start;

        for transform in transforms {
            if let Some(next) = self.transform(current, transform) {
                result.push(next);
                current = next;
            } else {
                // If transformation fails, stop here
                break;
            }
        }

        result
    }

    /// Generate a hexatonic cycle from a starting chord and pattern
    pub fn hexatonic_cycle(&self, start: Chord, pattern: &[Transform]) -> Option<Vec<Chord>> {
        let mut cycle = vec![start];
        let mut current = start;

        // Apply pattern repeatedly to generate 6 chords
        for i in 0..5 {
            let transform = &pattern[i % pattern.len()];
            current = self.transform(current, transform)?;
            cycle.push(current);
        }

        // Verify it cycles back to start
        let final_transform = &pattern[5 % pattern.len()];
        let should_be_start = self.transform(current, final_transform)?;

        if should_be_start == start { Some(cycle) } else { None }
    }

    /// Get all four standard hexatonic cycles
    pub fn all_hexatonic_cycles(&self) -> HashMap<&'static str, Vec<Vec<Chord>>> {
        let mut cycles = HashMap::new();

        // Pattern definitions
        let patterns = vec![
            ("Northern (PL)", vec![Transform::P, Transform::L]),
            ("Western (PR)", vec![Transform::P, Transform::R]),
            ("Eastern (LR)", vec![Transform::L, Transform::R]),
        ];

        for (name, pattern) in patterns {
            let mut cycle_group = Vec::new();
            let mut seen = std::collections::HashSet::new();

            // Try each chord as a starting point
            for chord in self.transformations.keys() {
                if seen.contains(chord) {
                    continue;
                }

                if let Some(cycle) = self.hexatonic_cycle(*chord, &pattern) {
                    // Mark all chords in this cycle as seen
                    for c in &cycle {
                        seen.insert(*c);
                    }
                    cycle_group.push(cycle);
                }
            }

            cycles.insert(name, cycle_group);
        }

        cycles
    }

    /// Transform any Transformable type using basic neo-Riemannian operations
    pub fn transform_any<T: Transformable>(&self, item: T, transform: &Transform) -> Option<T> {
        item.transform_basic(transform)
    }

    /// Apply P3,0 transformation to any Transformable type that supports it
    pub fn transform_p3_any<T: Transformable>(&self, item: T, transform: &P3Transform) -> Option<T> {
        item.transform_p3(transform)
    }

    /// Apply a sequence of transformations to any Transformable type
    pub fn apply_sequence_any<T: Transformable>(&self, start: T, transforms: &[Transform]) -> Vec<T> {
        let mut result = vec![start];
        let mut current = start;

        for transform in transforms {
            if let Some(next) = self.transform_any(current, transform) {
                result.push(next);
                current = next;
            } else {
                // If transformation fails, stop here
                break;
            }
        }

        result
    }
}

impl Default for NeoRiemannian {
    fn default() -> Self {
        Self::new()
    }
}

/// Voice leading optimizer for smooth transitions
pub struct VoiceLeading;

impl VoiceLeading {
    /// Calculate the voice leading distance between two chords
    pub fn distance(from: &[Note], to: &[Note]) -> i32 {
        if from.len() != to.len() {
            return i32::MAX;
        }

        from.iter()
            .zip(to.iter())
            .map(|(a, b)| {
                let diff = b.to_midi() as i32 - a.to_midi() as i32;
                diff.abs()
            })
            .sum()
    }

    /// Find the best inversion of the target chord for smooth voice leading
    pub fn optimize(from: &[Note], to_chord: Chord, octave: i8) -> Vec<Note> {
        let base_notes = to_chord.notes(octave);
        let mut best = base_notes.clone();
        let mut best_distance = Self::distance(from, &base_notes);

        // Try different inversions and octave adjustments
        for oct_adjust in -1..=1 {
            let adjusted = to_chord.notes(octave + oct_adjust);
            let distance = Self::distance(from, &adjusted);
            if distance < best_distance {
                best = adjusted;
                best_distance = distance;
            }
        }

        best
    }
}

/// Implementation of Transformable for the basic Chord type
impl Transformable for Chord {
    fn transform_basic(&self, transform: &Transform) -> Option<Self> {
        // For extended chords (sevenths, suspended), transform the underlying triad and
        // preserve extensions
        if !self.quality.is_triad() || self.quality.is_suspended() {
            return self.transform_extended(transform);
        }

        // For basic triads (major, minor, diminished, augmented), use the standard
        // neo-Riemannian engine
        let engine = NeoRiemannian::new();
        engine.transform(*self, transform)
    }

    fn transform_p3(&self, transform: &P3Transform) -> Option<Self> {
        if !self.quality.is_seventh() {
            return None;
        }

        // Implement P3,0 transformations for seventh chords
        self.transform_p3_seventh(transform)
    }

    fn notes(&self, octave: i8) -> Vec<Note> {
        self.notes(octave) // Delegate to existing method
    }

    fn underlying_triad(&self) -> Option<Chord> {
        // Always return the underlying triad (basic chord basis for transformations)
        Some(Chord::new(self.root, self.quality.underlying_triad()))
    }

    fn supports_p3_transforms(&self) -> bool {
        self.quality.is_seventh()
    }
}

impl Chord {
    /// Transform extended chords by transforming the underlying triad
    fn transform_extended(&self, transform: &Transform) -> Option<Self> {
        let underlying = self.underlying_triad()?;
        let engine = NeoRiemannian::new();
        let transformed_triad = engine.transform(underlying, transform)?;

        // Preserve the extension while updating the triad
        let new_quality = match self.quality {
            // Seventh chords: preserve seventh type, update triad basis
            ChordQuality::MajorSeventh => match transformed_triad.quality {
                ChordQuality::Major => ChordQuality::MajorSeventh,
                ChordQuality::Minor => ChordQuality::MinorMajorSeventh, // Keep major 7th
                ChordQuality::Diminished => ChordQuality::HalfDiminished,
                ChordQuality::Augmented => ChordQuality::AugmentedMajorSeventh,
                _ => return None,
            },
            ChordQuality::MinorSeventh => match transformed_triad.quality {
                ChordQuality::Major => ChordQuality::DominantSeventh,
                ChordQuality::Minor => ChordQuality::MinorSeventh,
                ChordQuality::Diminished => ChordQuality::HalfDiminished,
                _ => return None,
            },
            ChordQuality::DominantSeventh => match transformed_triad.quality {
                ChordQuality::Major => ChordQuality::DominantSeventh,
                ChordQuality::Minor => ChordQuality::MinorSeventh,
                ChordQuality::Diminished => ChordQuality::HalfDiminished,
                _ => return None,
            },
            ChordQuality::HalfDiminished => match transformed_triad.quality {
                ChordQuality::Major => ChordQuality::DominantSeventh,
                ChordQuality::Minor => ChordQuality::MinorSeventh,
                ChordQuality::Diminished => ChordQuality::HalfDiminished,
                _ => return None,
            },
            ChordQuality::FullyDiminished => match transformed_triad.quality {
                ChordQuality::Diminished => ChordQuality::FullyDiminished,
                _ => return None, // Fully diminished can only transform to other fully diminished
            },

            // Suspended chords: transform the resolved triad, keep suspension
            ChordQuality::Sus2 => match transformed_triad.quality {
                ChordQuality::Major => ChordQuality::Sus2,
                ChordQuality::Minor => ChordQuality::Sus2, // Allow sus2 over minor chords
                _ => return None,
            },
            ChordQuality::Sus4 => match transformed_triad.quality {
                ChordQuality::Major => ChordQuality::Sus4,
                ChordQuality::Minor => ChordQuality::Sus4, // Allow sus4 over minor chords
                _ => return None,
            },
            ChordQuality::SevenSus2 => match transformed_triad.quality {
                ChordQuality::Major => ChordQuality::SevenSus2,
                _ => return None,
            },
            ChordQuality::SevenSus4 => match transformed_triad.quality {
                ChordQuality::Major => ChordQuality::SevenSus4,
                _ => return None,
            },

            _ => return None,
        };

        Some(Chord::new(transformed_triad.root, new_quality))
    }

    /// P3,0 transformations for seventh chords
    /// Three voices move by half-step, one stays as common tone
    fn transform_p3_seventh(&self, transform: &P3Transform) -> Option<Self> {
        use PitchClass::*;

        match (self.root, &self.quality, transform) {
            // Major seventh chords - P3,0 transformations
            (C, ChordQuality::MajorSeventh, P3Transform::ParallelUp(CommonTone::Root)) => {
                Some(Chord::new(C, ChordQuality::MinorSeventh)) // C-E-G-B → C-Eb-Gb-Bb
            }
            (C, ChordQuality::MajorSeventh, P3Transform::ParallelUp(CommonTone::Third)) => {
                Some(Chord::new(Cs, ChordQuality::MinorSeventh)) // C-E-G-B → Db-E-Ab-B
            }
            (C, ChordQuality::MajorSeventh, P3Transform::ParallelUp(CommonTone::Fifth)) => {
                Some(Chord::new(Ds, ChordQuality::MinorSeventh)) // C-E-G-B → Db-F-G-C
            }
            (C, ChordQuality::MajorSeventh, P3Transform::ParallelUp(CommonTone::Seventh)) => {
                Some(Chord::new(F, ChordQuality::MinorSeventh)) // C-E-G-B → Db-F-Ab-B
            }

            // Minor seventh chords - P3,0 transformations
            (C, ChordQuality::MinorSeventh, P3Transform::ParallelUp(CommonTone::Root)) => {
                Some(Chord::new(C, ChordQuality::MajorSeventh)) // C-Eb-G-Bb → C-E-G#-B
            }
            (C, ChordQuality::MinorSeventh, P3Transform::ParallelDown(CommonTone::Root)) => {
                Some(Chord::new(C, ChordQuality::DominantSeventh)) // C-Eb-G-Bb → C-E-G-A
            }

            // Dominant seventh chords - P3,0 transformations
            (C, ChordQuality::DominantSeventh, P3Transform::ParallelUp(CommonTone::Root)) => {
                Some(Chord::new(C, ChordQuality::MinorMajorSeventh)) // C-E-G-Bb → C-Eb-Ab-B
            }
            (C, ChordQuality::DominantSeventh, P3Transform::ParallelDown(CommonTone::Seventh)) => {
                Some(Chord::new(Fs, ChordQuality::HalfDiminished)) // C-E-G-Bb → B-D-F-A
            }

            // For now, implement a subset - this could be expanded to all 12 chromatic roots
            // and all seventh chord types with proper voice leading calculations
            _ => {
                // For unimplemented combinations, try a basic transformation approach
                // Transform the underlying triad and adjust the seventh accordingly
                if let Some(underlying) = self.underlying_triad() {
                    let engine = NeoRiemannian::new();
                    // Use a basic P transform as a fallback
                    let transformed_triad = engine.transform(underlying, &Transform::P)?;

                    // Try to preserve the seventh chord character
                    let new_quality = match self.quality {
                        ChordQuality::MajorSeventh => match transformed_triad.quality {
                            ChordQuality::Minor => ChordQuality::MinorMajorSeventh,
                            _ => ChordQuality::MajorSeventh,
                        },
                        ChordQuality::MinorSeventh => match transformed_triad.quality {
                            ChordQuality::Major => ChordQuality::DominantSeventh,
                            _ => ChordQuality::MinorSeventh,
                        },
                        ChordQuality::DominantSeventh => match transformed_triad.quality {
                            ChordQuality::Minor => ChordQuality::MinorSeventh,
                            _ => ChordQuality::DominantSeventh,
                        },
                        _ => return None,
                    };

                    Some(Chord::new(transformed_triad.root, new_quality))
                } else {
                    None
                }
            }
        }
    }
}

/// Mode-constrained neo-Riemannian transformation engine
/// Restricts transformations to chords that exist within a specified mode
pub struct ModalNeoRiemannian {
    mode: Mode,
    tonic: PitchClass,
    valid_chords: HashSet<Chord>,
    #[allow(dead_code)]
    base_transformer: NeoRiemannian,
    constrained_transformations: HashMap<Chord, TransformMap>,
}

impl ModalNeoRiemannian {
    /// Create a new modal neo-Riemannian transformer
    pub fn new(mode: Mode, tonic: PitchClass) -> Self {
        let valid_chords: HashSet<Chord> = mode.triads(tonic).into_iter().collect();
        let base_transformer = NeoRiemannian::new();

        // Build constrained transformation map
        let mut constrained_transformations = HashMap::new();

        for &chord in &valid_chords {
            if let Some(base_transforms) = base_transformer.transformations.get(&chord) {
                let constrained_map = TransformMap {
                    p: Self::find_best_substitute(&valid_chords, base_transforms.p, chord),
                    r: Self::find_best_substitute(&valid_chords, base_transforms.r, chord),
                    l: Self::find_best_substitute(&valid_chords, base_transforms.l, chord),
                };
                constrained_transformations.insert(chord, constrained_map);
            }
        }

        ModalNeoRiemannian {
            mode,
            tonic,
            valid_chords,
            base_transformer,
            constrained_transformations,
        }
    }

    /// Find the best substitute chord within the mode if the target is invalid
    fn find_best_substitute(valid_chords: &HashSet<Chord>, target: Chord, fallback: Chord) -> Chord {
        if valid_chords.contains(&target) {
            return target;
        }

        // Try to find the chord with the same root but different quality
        let same_root_chords: Vec<&Chord> = valid_chords.iter().filter(|c| c.root == target.root).collect();

        if let Some(&chord) = same_root_chords.first() {
            return *chord;
        }

        // Try to find a chord with similar harmonic function
        // If target was major, prefer major chords; if minor, prefer minor
        let preferred_quality = target.quality;
        let similar_quality_chords: Vec<&Chord> =
            valid_chords.iter().filter(|c| c.quality == preferred_quality).collect();

        if let Some(&chord) = similar_quality_chords.first() {
            return *chord;
        }

        // As a last resort, stay on the current chord
        fallback
    }

    /// Apply a transformation within the modal constraints
    pub fn transform(&self, chord: Chord, transform: &Transform) -> Option<Chord> {
        // First check if the chord is valid in this mode
        if !self.valid_chords.contains(&chord) {
            return None;
        }

        match transform {
            Transform::P => self.constrained_transformations.get(&chord).map(|t| t.p),
            Transform::R => self.constrained_transformations.get(&chord).map(|t| t.r),
            Transform::L => self.constrained_transformations.get(&chord).map(|t| t.l),
            Transform::Compound(transforms) => {
                let mut current = chord;
                for t in transforms {
                    current = self.transform(current, t)?;
                }
                Some(current)
            }
        }
    }

    /// Apply a sequence of transformations within modal constraints
    pub fn apply_sequence(&self, start: Chord, transforms: &[Transform]) -> Vec<Chord> {
        let mut result = vec![start];
        let mut current = start;

        for transform in transforms {
            if let Some(next) = self.transform(current, transform) {
                result.push(next);
                current = next;
            } else {
                // If transformation fails, stop here
                break;
            }
        }

        result
    }

    /// Get all valid chords in this mode
    pub fn valid_chords(&self) -> &HashSet<Chord> {
        &self.valid_chords
    }

    /// Get the mode and tonic
    pub fn mode_info(&self) -> (Mode, PitchClass) {
        (self.mode, self.tonic)
    }

    /// Try to generate a modal hexatonic-like cycle (may not always close)
    pub fn modal_cycle(&self, start: Chord, pattern: &[Transform], max_length: usize) -> Vec<Chord> {
        let mut cycle = vec![start];
        let mut current = start;
        let mut seen = HashSet::new();
        seen.insert(start);

        for i in 0..max_length {
            let transform = &pattern[i % pattern.len()];
            if let Some(next) = self.transform(current, transform) {
                if seen.contains(&next) {
                    // We've created a cycle or hit a repeated chord
                    cycle.push(next);
                    break;
                }
                cycle.push(next);
                seen.insert(next);
                current = next;
            } else {
                break;
            }
        }

        cycle
    }

    /// Analyze the mode's harmonic relationships
    pub fn analyze_mode(&self) -> ModalAnalysis {
        let chords = self.mode.triads(self.tonic);
        let mut analysis = ModalAnalysis {
            mode: self.mode,
            tonic: self.tonic,
            chords: chords.clone(),
            primary_chords: Vec::new(),
            secondary_chords: Vec::new(),
            characteristic_chords: Vec::new(),
        };

        // Classify chords by their harmonic function
        for (i, chord) in chords.iter().enumerate() {
            match i {
                0 | 3 | 4 => analysis.primary_chords.push(*chord), // I, IV, V (tonic function area)
                1 | 2 | 5 => analysis.secondary_chords.push(*chord), // ii, iii, vi
                6 => analysis.characteristic_chords.push(*chord),  // vii°
                _ => {}
            }
        }

        // Add mode-specific characteristic chords
        match self.mode {
            Mode::Lydian => {
                // The #IV chord is characteristic of Lydian
                if chords.len() > 3 {
                    analysis.characteristic_chords.push(chords[3]);
                }
            }
            Mode::Mixolydian => {
                // The bVII chord is characteristic of Mixolydian
                if chords.len() > 6 {
                    analysis.characteristic_chords.push(chords[6]);
                }
            }
            Mode::Dorian => {
                // The IV chord (major) is characteristic of Dorian vs Aeolian
                if chords.len() > 3 {
                    analysis.characteristic_chords.push(chords[3]);
                }
            }
            _ => {}
        }

        analysis
    }

    /// Transform any Transformable type within modal constraints
    pub fn transform_any<T: Transformable>(&self, item: T, transform: &Transform) -> Option<T> {
        // For extended chords, we need to check if the underlying triad is valid
        let original_underlying = if let Some(underlying) = item.underlying_triad() {
            if !self.valid_chords.contains(&underlying) {
                return None;
            }
            underlying
        } else {
            // For items without underlying triads, we can't apply modal constraints
            return item.transform_basic(transform);
        };

        // Apply the transformation
        let result = item.transform_basic(transform)?;

        // Check if the result is valid in this mode (for basic triads)
        if let Some(result_triad) = result.underlying_triad() {
            if self.valid_chords.contains(&result_triad) {
                Some(result)
            } else {
                // Try to find a substitute within the mode
                let substitute = Self::find_best_substitute(&self.valid_chords, result_triad, original_underlying);
                if substitute == result_triad {
                    Some(result)
                } else {
                    // Return the original item if no good substitute is found
                    Some(item)
                }
            }
        } else {
            // For types without underlying triads, just return the result
            Some(result)
        }
    }

    /// Apply a sequence of transformations to any Transformable type within
    /// modal constraints
    pub fn apply_sequence_any<T: Transformable>(&self, start: T, transforms: &[Transform]) -> Vec<T> {
        let mut result = vec![start];
        let mut current = start;

        for transform in transforms {
            if let Some(next) = self.transform_any(current, transform) {
                result.push(next);
                current = next;
            } else {
                // If transformation fails, stop here
                break;
            }
        }

        result
    }
}

/// Analysis of a mode's harmonic structure
#[derive(Debug, Clone)]
pub struct ModalAnalysis {
    pub mode: Mode,
    pub tonic: PitchClass,
    pub chords: Vec<Chord>,
    pub primary_chords: Vec<Chord>,        // Main tonic-function chords
    pub secondary_chords: Vec<Chord>,      // Supporting chords
    pub characteristic_chords: Vec<Chord>, // Mode-defining chords
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChordQuality;
    use crate::core::PitchClass;

    #[test]
    fn basic_transformations() {
        let nr = NeoRiemannian::new();

        // P transformation: C major -> C minor
        let c_major = Chord::c_major();
        let c_minor = nr
            .transform(c_major, &Transform::P)
            .expect("test transform should work");
        assert_eq!(c_minor.quality, ChordQuality::Minor);
        assert_eq!(c_minor.root, PitchClass::C);

        // R transformation: C major -> A minor
        let a_minor = nr
            .transform(c_major, &Transform::R)
            .expect("test transform should work");
        assert_eq!(a_minor, Chord::a_minor());

        // L transformation: C major -> E minor
        let e_minor = nr
            .transform(c_major, &Transform::L)
            .expect("test transform should work");
        assert_eq!(e_minor, Chord::e_minor());
    }

    #[test]
    fn compound_transformations() {
        let nr = NeoRiemannian::new();
        let c_major = Chord::c_major();

        // RP: C major -> A minor -> A major
        let result = nr
            .transform(c_major, &Transform::rp())
            .expect("test transform should work");
        assert_eq!(result, Chord::major(PitchClass::A));
    }

    #[test]
    fn hexatonic_cycles() {
        let nr = NeoRiemannian::new();

        // Test PL cycle starting from C major
        let pattern = vec![Transform::P, Transform::L];
        let cycle = nr
            .hexatonic_cycle(Chord::c_major(), &pattern)
            .expect("test hexatonic cycle should work");

        // Should have 6 chords
        assert_eq!(cycle.len(), 6);

        // Verify it cycles back
        let last = cycle.last().expect("test cycle should have members");
        let back_to_start = nr.transform(*last, &pattern[1]).expect("test transform should work");
        assert_eq!(back_to_start, cycle[0]);
    }
}
