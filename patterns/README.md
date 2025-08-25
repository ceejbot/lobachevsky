# Rhythm Pattern Library 🎵

This directory contains 24+ high-quality rhythm patterns designed for flow-state and programming music. Each pattern is defined in TOML format and can be loaded dynamically at runtime.

## Pattern Categories

### 🧠 Flow State / Programming Music
Perfect for long coding sessions with hypnotic, non-jarring grooves:
- **deep_hypnotic** (126 BPM) - Extended focus sessions with minimal evolution
- **coding_flow** (120 BPM) - Steady, non-distracting groove for programming
- **focus_minimal** (118 BPM) - Stripped down for deep concentration  
- **ambient_progressive** (110 BPM) - Slow builds with atmospheric textures
- **trance_builder** (132 BPM) - Progressive builds without drops

### 🕺 Electronic Dance Subgenres
Authentic dance music patterns with proper groove:
- **deep_house** (125 BPM) - Classic swing, ghost notes, warm groove
- **tech_house** (128 BPM) - Driving 4/4 with syncopated percussion
- **progressive_house** (124 BPM) - Long builds for extended listening
- **minimal_techno** (130 BPM) - Hypnotic repetition with subtle variations
- **dub_techno** (125 BPM) - Spacious, reverb-heavy with delays
- **ambient_techno** (115 BPM) - Ethereal, slow-moving textures

### 🔬 Abstract / Experimental
Mathematical beauty and complex rhythmic patterns:
- **polyrhythmic_ambient** (108 BPM) - Multiple time signatures simultaneously
- **glitch_minimal** (122 BPM) - Stuttering patterns with micro-timing
- **algorithmic_abstract** (116 BPM) - Fibonacci sequences and golden ratio
- **drone_rhythmic** (88 BPM) - Very slow, meditative textures

### 🎛️ Classic Electronic & Functional
Traditional electronic styles plus tempo-specific work music:
- **breaks_dnb** (175 BPM) - Amen break variations and drum-n-bass
- **trip_hop** (95 BPM) - Heavy swing and laid-back groove
- **slow_hypnotic** (95 BPM) - Deep focus for extended concentration
- **medium_flow** (115 BPM) - Balanced energy for general programming
- **uptempo_coding** (130 BPM) - High energy without distraction
- **idm_complex** (140 BPM) - Intricate patterns inspired by IDM

## Usage

### Basic Pattern Loading

```rust
use lobachevsky::rhythm::PatternLibrary;

// Create library and load patterns
let mut library = PatternLibrary::new();
library.load_from_directory(std::path::Path::new("patterns"))?;

// Get a specific pattern
let pattern_data = library.get("deep_hypnotic").unwrap();
let rhythm_pattern = pattern_data.to_pattern()?;

// Generate events for a bar
let events = rhythm_pattern.events_for_bar(0);
```

### Programmatic Pattern Creation

```rust
use lobachevsky::rhythm::{SwingPattern, EuclideanPattern, DrumVoice};

// Create a swing pattern programmatically
let base = Box::new(EuclideanPattern::new(DrumVoice::Kick, 4, 4));
let swing = SwingPattern::new(base, 0.67, 0.25) // 67% swing on 16th notes
    .with_swing_accent(10); // +10 velocity on off-beats
```

## Advanced Features

### 🎯 Pattern Types

**Euclidean Patterns**
```toml
[layers.pattern_type]
type = "Euclidean"
hits = 5        # Number of hits to distribute
steps = 8       # Total time steps  
rotation = 2    # Optional: rotate pattern
velocity = 70   # MIDI velocity
```

**Probability Patterns**
```toml
[layers.pattern_type]
type = "Probability"
points = [
    { beat = 0.0, probability = 0.9 },
    { beat = 1.5, probability = 0.6 },
]
velocity_range = [40, 80]  # Random velocity range
```

**Swing Patterns**
```toml
[layers.pattern_type]
type = "Swing"
swing_ratio = 0.67      # 0.5 = straight, 0.67 = heavy swing
subdivision = 0.25      # 16th notes
swing_accent = 8        # Optional velocity boost for off-beats
[layers.pattern_type.base_pattern]
type = "Euclidean"
hits = 4
steps = 8
```

**Polyrhythmic Patterns**
```toml
[layers.pattern_type]
type = "Polyrhythmic"
time_signature = [7, 8]    # 7/8 time
pattern_length = 2         # Bars before repeat
[layers.pattern_type.base_pattern]
type = "Euclidean"
hits = 5
steps = 7
```

**Groove Patterns**
```toml
[layers.pattern_type]
type = "Groove"
intensity = 0.6            # Groove effect strength
humanization = 0.02        # Random timing variation

# Template groove
[layers.pattern_type.groove_type]
groove_type = "Template"
name = "deep_house_shuffle"

# OR custom groove with precise timing
[layers.pattern_type.groove_type]
groove_type = "Custom"
[layers.pattern_type.groove_type.timing_map]
"0.0" = 0.01    # Beat 0: +0.01 beat timing adjustment
"1.0" = -0.005  # Beat 1: -0.005 beat timing adjustment
```

### 🥁 Available Drum Voices
- **kick**, **kick_soft** - Main and ghost kicks (GM 36, 35)
- **snare**, **rim**, **clap** - Snare family (GM 38, 37, 39)
- **hihat_closed**, **hihat_open** - Hi-hats (GM 42, 46)
- **shaker**, **ride**, **percussion** - Texture elements (GM 70, 51, 69)

### 🎼 Complete Pattern Example

```toml
name = "example_flow_pattern"
description = "Perfect for deep work sessions"
tempo_hint = 120

# Solid kick foundation
[[layers]]
voice = "kick"
[layers.pattern_type]
type = "Groove"
intensity = 0.4
[layers.pattern_type.base_pattern]
type = "Euclidean"
hits = 4
steps = 4
velocity = 75
[layers.pattern_type.groove_type]
groove_type = "Template"
name = "deep_house_shuffle"

# Swinging hi-hats
[[layers]]
voice = "hihat_closed"
[layers.pattern_type]
type = "Swing"
swing_ratio = 0.62
subdivision = 0.125
[layers.pattern_type.base_pattern]
type = "Euclidean"
hits = 8
steps = 16
velocity = 40

# Polyrhythmic texture
[[layers]]
voice = "ride"
[layers.pattern_type]
type = "Polyrhythmic"
time_signature = [5, 4]
pattern_length = 4
[layers.pattern_type.base_pattern]
type = "Probability"
points = [
    { beat = 0.0, probability = 0.7 },
    { beat = 1.2, probability = 0.5 },
]
velocity_range = [25, 35]
```

## Integration with CLI

Use patterns with the ambient rhythm generator:

```bash
# Generate using pattern library (future feature)
lobachevsky ambient --pattern deep_hypnotic --bars 64

# Or use built-in styles
lobachevsky ambient --style minimal --bars 32 --tempo 128
```

## Mathematical Patterns

Several patterns use mathematical sequences for algorithmic beauty:
- **algorithmic_abstract** - Fibonacci sequences (5, 8, 13, 21 steps)
- **polyrhythmic_ambient** - Prime time signatures (7/8, 11/16, 17/16)
- Golden ratio timing adjustments (0.618, 1.618)

## Pattern Design Philosophy

These patterns are designed specifically for:
- **Flow state induction** - Hypnotic, non-jarring progressions
- **Extended listening** - No sudden drops or transitions
- **Programming focus** - Steady energy without distraction
- **Mathematical beauty** - Algorithmic precision and elegance
- **Gradual evolution** - Subtle changes over time

Perfect for long coding sessions, deep work, and meditative programming! 🎧✨
