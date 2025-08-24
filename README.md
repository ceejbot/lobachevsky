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
```

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
