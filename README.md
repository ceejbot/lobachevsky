# lobachevsky - Neo-Riemannian music theory library

A Rust library for exploring neo-Riemannian music theory through generative composition, providing tools for chord transformations, rhythm patterns, euclidian rhythms, and MIDI output. боже мой!

## Features

- **A reasonable selection of musical primitives**: Representations of notes, chords, and pitch classes
- **Neo-Riemannian transformations**: has complete P/R/L transformation mappings for all 24 major/minor triads
- **Hexatonic cycles**: Generate pieces with Northern (PL), Western (PR), and Eastern (LR) cycles
- **Generalized rhythm system**: Trait-based pattern generation with multiple strategies:
  - Euclidean rhythms for mathematical distribution
  - Probability-based patterns with variation
  - Genre-specific templates (90s ambient, 2000s microhouse, 2010s Euclidean)
  - Layered composition for complex rhythms
- **Voice leading optimization**: smoothly transitions between chords
- **MIDI output**: generates standard MIDI files
- **Builder pattern APIs**: reasonable interfaces for complex compositions

## CLI Usage

```text
> lobachevsky --help
A library and command-line tool for generating neo-Reimannian progressions as midi files

Usage: lobachevsky <COMMAND>

Commands:
  progression  Generate a neo-Riemannian progression
  hexatonic    Explore hexatonic cycles
  ambient      Generate ambient techno composition
  compose      Generate a complete composition with harmony, rhythm, and melody
  modal        Explore modal neo-Riemannian transformations
  extended     Explore extended chord transformations (seventh chords, suspended chords)
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```


### Generating progressions

```bash
❯ lobachevsky progression --help
Generate a neo-Riemannian progression

Usage: lobachevsky progression [OPTIONS] [OUTPUT]

Arguments:
  [OUTPUT]  Where to store the generated MIDI [default: generated_progression.mid]

Options:
  -s, --start <START>      Starting chord (e.g., "F", "Am", "C#m") [default: F]
  -p, --pattern <PATTERN>  Transformation pattern (P, R, L) [default: P,R,L]
  -l, --length <LENGTH>    Number of chords in progression [default: 8]
  -r, --return-to-start    Return to starting chord
  -m, --mode <MODE>        Modal constraint (e.g., "lydian", "dorian", "mixolydian")
  -t, --tonic <TONIC>      Modal tonic (e.g., "C", "F#", "Bb") - required if mode is specified
  -h, --help               Print help

# Basic progression from F major
lobachevsky progression

# Custom pattern with return to start
lobachevsky progression --start C --pattern "P,R,L,R" --length 16 --return-to-start

# Starting from A minor with compound transformations
lobachevsky progression --start Am --pattern "PL,PR,RL" - 12 a_minor_compound.mid
```

### Generating hexatonic cycles

There are three hexatonic cycles. This generator is less elaborate.

```bash
❯ lobachevsky hexatonic --help
Explore hexatonic cycles

Usage: lobachevsky hexatonic [OPTIONS] [OUTPUT]

Arguments:
  [OUTPUT]  Where to store the generated MIDI [default: generated_progression.mid]

Options:
  -c, --cycle <CYCLE>  Cycle type: northern, western, eastern [default: northern]
  -s, --start <START>  Starting chord [default: C]
  -h, --help           Print help

# Northern cycle from C major
lobachevsky hexatonic --cycle northern --start C
```

### Generate ambient techno rhythms

I was interested in auto-generating some varying rhythms in software instead of with my hardware sequence, just to see how difficult this stuff is. Survey says: not all that difficult. I'll probably go nuts with this.

```bash
# 90s ambient style
lobachevsky ambient --style 90s --bars 64 --tempo 110

# 2000s microhouse
lobachevsky ambient --style 2000s --bars 128 --tempo 124

# 2010s Euclidean patterns
lobachevsky ambient --style 2010s --bars 64 --tempo 120

# Detroit techno - raw, driving
lobachevsky ambient --style detroit --bars 32 --tempo 132

# Berlin dub techno - deep, atmospheric
lobachevsky ambient --style berlin --bars 64 --tempo 125

# Minimal Berlin - stripped-down, hypnotic
lobachevsky ambient --style minimal --bars 48 --tempo 128

# Acid minimal - 303-influenced with syncopation
lobachevsky ambient --style acid --bars 64 --tempo 135

# Deep minimal - subdued with ghost notes
lobachevsky ambient --style deep --bars 32 --tempo 120
```

## Rhythm Styles

The rhythm generator supports multiple styles, each with distinct characteristics:

### Era-Based Styles
- **90s**: Sparse, breathing ambient techno with subtle hi-hat patterns and minimal snare placement
- **2000s**: Shuffled microhouse with ghost notes and soft kick variations
- **2010s**: Complex Euclidean patterns with mathematical rhythm distribution

### Regional/Genre Styles
- **detroit**: Raw, driving 4/4 with boomy kicks and sparse percussion, emphasizing the off-beats
- **berlin**: Deep, atmospheric dub techno with spacious arrangements and delayed elements
- **minimal**: Stripped-down, hypnotic patterns focusing on rolling hi-hat grooves
- **acid**: 303-influenced patterns with syncopated percussion and complex polyrhythmic elements
- **deep**: Subdued patterns emphasizing ghost notes and subtle swing for underground vibes

Each style uses different combinations of:
- **EuclideanPattern**: Mathematical distribution of hits across time steps
- **ProbabilityPattern**: Stochastic variations with per-beat hit probabilities
- **LayeredPattern**: Multiple synchronized rhythm layers

### Create complete compositions

Hey, why not. I too want to do CCRMA concerts circa 1993.

```bash
# Generate full composition with harmony, rhythm, and melody
lobachevsky compose --bars 64 --tempo 110 --output my_composition.mid
```

## The lobachevsky library

Consult `cargo doc --open` to read the library API documentation.

### Architecture

```text
src
├── core.rs        # core music types
├── generation.rs  # progression and melody generation tools
├── lib.rs         #
├── main.rs        # cli implementation
├── midi.rs        # midi output
├── rhythm.rs      # rhythm layers, patterns, and generators
└── theory.rs      # types and implementations for neo-Reimannian xforms etc
```

The core music types are:

- `PitchClass`: Enum representing the 12 pitch classes
- `Note`: Specific pitch with octave
- `Chord`, `ChordQuality`: Root + quality (major/minor/diminished/augmented/etc)
- `Mode`: the mode of a scale
- `Transformable`: a trait

The important theory types are:

- `Transform`: Neo-Riemannian transformations of triads (P, R, L, and compounds)
- `NeoRiemannian`: a map of transformations precalculated for triads
- `ModalNeoRiemannian`: a map of *modally constrained* transformations precalculated for triads

The rhythm system uses a trait-based approach allowing multiple pattern generation strategies:

```rust
pub trait RhythmPattern {
    fn events_for_bar(&self, bar: usize) -> Vec<DrumEvent>;
    fn pattern_length(&self) -> usize;
    fn clone_box(&self) -> Box<dyn RhythmPattern>;
}
```

Implementations include:

- `EuclideanPattern`: Mathematical rhythm distribution
- `ProbabilityPattern`: Stochastic variations
- `LayeredPattern`: Combine multiple patterns
- `GenrePatterns`: Pre-built genre-specific templates

## Pattern Format

The library supports loading rhythm patterns from TOML files for data-driven pattern creation. Pattern files use the following structure:

### Basic Structure

```toml
name = "pattern_name"
description = "Optional description of the pattern"
tempo_hint = 128  # Optional suggested BPM

[[layers]]
voice = "kick"
[layers.pattern_type]
pattern_type = "Euclidean"
hits = 4
steps = 16
rotation = 0    # Optional
velocity = 80   # Optional
```

### Pattern Types

#### Euclidean Patterns
Mathematical distribution of hits across time steps:
```toml
[layers.pattern_type]
pattern_type = "Euclidean"
hits = 5        # Number of hits to distribute
steps = 8       # Total time steps
rotation = 2    # Optional: rotate pattern
velocity = 70   # Optional: MIDI velocity
```

#### Probability Patterns
Stochastic patterns with per-beat hit probabilities:
```toml
[layers.pattern_type]
pattern_type = "Probability"
velocity_range = [40, 80]  # Optional: min/max velocity range
[[layers.pattern_type.points]]
beat = 0.0
probability = 0.9
[[layers.pattern_type.points]]
beat = 1.5
probability = 0.6
```

### Valid Drum Voices
- `kick`, `kick_soft`: Kick drums (GM notes 36, 35)
- `snare`, `rim`, `clap`: Snare family (GM notes 38, 37, 39)
- `hihat_closed`, `hihat_open`: Hi-hats (GM notes 42, 46)
- `shaker`, `ride`, `percussion`: Other percussion (GM notes 70, 51, 69)

### Example: UK Garage Pattern
```toml
name = "uk_garage"
description = "UK Garage with shuffled hi-hats and syncopated kicks"
tempo_hint = 138

[[layers]]
voice = "kick"
[layers.pattern_type]
pattern_type = "Probability"
points = [
    { beat = 0.0, probability = 1.0 },
    { beat = 1.5, probability = 0.8 },
    { beat = 2.0, probability = 0.9 },
    { beat = 3.5, probability = 0.7 },
]
velocity_range = [80, 90]

[[layers]]
voice = "snare"
[layers.pattern_type]
pattern_type = "Probability"
points = [
    { beat = 1.0, probability = 0.9 },
    { beat = 3.0, probability = 0.95 },
]

[[layers]]
voice = "hihat_closed"
[layers.pattern_type]
pattern_type = "Euclidean"
hits = 13
steps = 16
rotation = 2
velocity = 45
```

Load patterns using the `PatternLibrary` API:
```rust
use lobachevsky::rhythm::{PatternLibrary, PatternData};

let mut library = PatternLibrary::new();
library.load_from_directory(Path::new("patterns"))?;

let pattern = library.get("uk_garage").unwrap();
let rhythm = pattern.to_pattern()?;
```

### Extensions

I'm thinking about extending the library in a few ways, wherever my music theory exploration urges take me:

- add new transformation types
- implement custom rhythm patterns (very likely)
- create new melody generation strategies (once I learn about them)
- add support for microtonality (maybe; future)
- design a text-based composition format for complex arrangements
- implement some advanced voice leading algorithms once I learn about them
- do real-time MIDI output (maybe)
- integrate with hardware synthesizers (maybe; send midi directly)
- do machine learning for pattern generation (maybe)

## Development

This is a Rust project using the 2024 edition. There's a `.justfile` with conveniences, but all the usual Rust & cargo commands do the usual.

## About the name

One man deserves the credit; one man deserves the blame; and Nikolai Ivanovich Lobachevsky is his name oi! Reimannian Euclidian thingies are both involved. How could I not? RIP Tom.

## LICENSE

This code is licensed via [the Parity Public License.](https://paritylicense.com) This license requires people who build on top of this source code to share their work with the community, too. This means if you hack on it for work, you have to make your work repo public somehow. I mean, have fun. See the license text for details.
