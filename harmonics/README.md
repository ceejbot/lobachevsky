# Harmonic Pattern Library 🎼

This directory contains TOML-defined harmonic progressions for use with the `generate` command.

## Usage

Use harmonic patterns with the generate command:

```bash
# Use a named pattern from this directory
lobachevsky generate --harmony dorian_journey --rhythm deep_hypnotic --melody arpeggio

# Use a specific TOML file
lobachevsky generate --harmony harmonics/lydian_exploration.toml --rhythm coding_flow

# Or define harmony inline
lobachevsky generate --start Am --pattern "P,L,R" --rhythm minimal_techno
```

## Pattern Format

```toml
name = "pattern_name"
description = "Description of the harmonic progression"
start_chord = "C"  # Starting chord (C, Am, F#m, etc.)
transformations = ["P", "L", "R"]  # Neo-Riemannian transformations
mode = "dorian"  # Optional: modal constraint
tonic = "D"      # Required if mode is specified
return_to_start = true  # Whether to return to starting chord
tempo_hint = 120  # Suggested tempo
bars_hint = 32   # Suggested number of bars
```

## Available Patterns

### Classic Neo-Riemannian
- **classic_plr** - Traditional P-L-R transformation sequence
- **compound_transformations** - Using compound transforms (RP, PL, LR, etc.)

### Modal Explorations
- **dorian_journey** - D Dorian modal progression
- **lydian_exploration** - F Lydian exploration with neo-Riemannian moves

### Hexatonic Cycles
- **hexatonic_northern** - Northern hexatonic cycle using PL transforms

### Ambient/Atmospheric
- **ambient_flow** - Slow, atmospheric progression for ambient music

## Transformation Reference

### Basic Transformations
- **P** (Parallel) - Major ↔ Minor on same root (C → Cm)
- **R** (Relative) - Major ↔ Relative minor (C → Am)
- **L** (Leading-tone) - Exchange via leading tone (C → Em)

### Compound Transformations
- **RP** - Relative then Parallel
- **PL** - Parallel then Leading-tone
- **LR** - Leading-tone then Relative
- **PLP** - Three-step transformation
- **RPL** - Three-step transformation
- **LPL** - Three-step transformation

## Creating Your Own Patterns

1. Create a new `.toml` file in this directory
2. Define the pattern using the format above
3. Use it with: `lobachevsky generate --harmony your_pattern_name`

## Integration with Rhythm Patterns

Harmonic patterns work seamlessly with rhythm patterns from the `patterns/` directory:

```bash
# Ambient composition
lobachevsky generate \
  --harmony ambient_flow \
  --rhythm deep_hypnotic_alpha \
  --melody stepwise \
  --bars 64

# Upbeat coding music
lobachevsky generate \
  --harmony dorian_journey \
  --rhythm uptempo_coding \
  --melody mixed \
  --tempo 130
```

## Tips for Pattern Design

1. **Bar Alignment**: The generator will automatically align to 16/32 bar multiples
2. **Return to Start**: Enable for better loop points in DAWs
3. **Modal Constraints**: Keep progressions within a specific mode/scale
4. **Tempo Hints**: Match harmonic rhythm to suggested tempo
5. **Transformation Length**: 6-8 transformations work well for 32 bars