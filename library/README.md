# Lobachevsky Pattern Library

This library contains comprehensive patterns, templates, and sound design guidance for electronic music generation using the Lobachevsky system.

## Library Structure

```
library/
├── patterns/          # Drum and percussion patterns with voicing suggestions
├── bass/             # Bass pattern libraries with synthesis guidance  
├── harmonics/        # Harmonic progressions using neo-Riemannian theory
├── styles/           # Complete style templates combining all elements
└── README.md         # This documentation
```

## Style Templates

Style templates provide complete production guidelines for specific electronic music genres. Each template includes:

- **Pattern Recommendations**: Which drum, bass, and harmony patterns work best
- **Synthesis Guidance**: Detailed sound design instructions  
- **Mixing Philosophy**: Frequency balance, effects usage, spatial design
- **Arrangement Structure**: Typical song structures and build techniques
- **Historical Context**: Artists, labels, tracks, and hardware references

### Available Style Templates

#### `minimal_techno_berlin.toml`
- **Tempo**: 128-135 BPM (preferred: 132)
- **Philosophy**: Surgical precision meets hypnotic depth
- **Key Elements**: Clean kick, minimal percussion, space and precision
- **References**: Robert Hood, Plastikman, Basic Channel
- **Hardware**: TR-909, TB-303, Access Virus

#### `ambient_techno_detroit.toml` 
- **Tempo**: 110-120 BPM (preferred: 115)
- **Philosophy**: Technology with soul - electronic music with human warmth
- **Key Elements**: Warm pads, musical bass lines, organic groove
- **References**: Carl Craig, Moodymann, Underground Resistance
- **Hardware**: Juno-106, DX7, MPC 3000

#### `uk_breakbeat_hardcore.toml`
- **Tempo**: 150-180 BPM (preferred: 165) 
- **Philosophy**: Raw energy and analog chaos - music for altered states
- **Key Elements**: Chopped Amen breaks, Reese bass, analog distortion
- **References**: The Prodigy, Altern-8, early UK rave scene
- **Hardware**: Akai MPC60, TB-303, analog samplers

#### `deep_dub_techno.toml`
- **Tempo**: 120-128 BPM (preferred: 124)
- **Philosophy**: Space and depth - music for contemplation and deep listening  
- **Key Elements**: Delay networks, sustained chords, infinite depth
- **References**: Basic Channel, Rhythm & Sound, Deepchord
- **Hardware**: Roland Space Echo, Eventide delays, Juno-106

#### `uplifting_trance_goa.toml`
- **Tempo**: 132-138 BPM (preferred: 135)
- **Philosophy**: Euphoric journey - music for transcendence and unity
- **Key Elements**: Supersaw leads, rolling basslines, emotional chord progressions
- **References**: Astrix, Paul van Dyk, classic uplifting trance
- **Hardware**: Access Virus TI, JP-8000, Novation SuperNova

## Pattern Categories

### Drum Patterns (`patterns/`)

Each drum pattern includes comprehensive voicing suggestions:

- **Synthesis Type**: Analog, digital, sampled, or hybrid approaches
- **Processing Chain**: EQ, compression, effects, and spatial placement
- **Character Description**: The sonic personality and role of each element
- **Hardware/Software References**: Specific gear recommendations
- **Mixing Notes**: How elements sit in the frequency spectrum and stereo field

#### Style-Specific Examples:
- `minimal_techno.toml` - Mathematical precision with subtle human touches
- `ambient_techno.toml` - Organic, spacious textures with subtle electronic elements  
- `tech_house.toml` - Punchy, groove-focused with tight rhythmic precision
- `breakbeat.toml` - Gritty, sample-based with analog warmth
- `dub_techno_deep.toml` - Spacious, delay-heavy with infinite depth

### Bass Patterns (`bass/`)

Bass patterns provide both rhythmic patterns and complete synthesis guidance:

- **Synthesis Approach**: Analog, digital, or hybrid synthesis methods
- **Waveform Selection**: Specific waveforms and harmonic content
- **Filter Design**: Filter types, cutoff frequencies, and resonance settings
- **Envelope Shaping**: Attack, decay, sustain, and release characteristics
- **Effects Processing**: Distortion, compression, and spatial effects
- **Production Notes**: Mixing approach and frequency considerations

#### Available Bass Patterns:
- `techno_acid_bass.toml` - Classic TB-303 style acid bass with resonant filtering
- `house_classic_bass.toml` - Warm Moog-style bass with musical character
- `minimal_techno_bass.toml` - Clean, focused bass with surgical precision  
- `ambient_organic_bass.toml` - Warm, evolving bass with organic character
- `breakbeat_reese_bass.toml` - Deep, gritty Reese bass with harmonic distortion
- `dub_techno_deep_bass.toml` - Deep, sustained bass with analog warmth
- `trance_rolling_bass.toml` - Bright, driving bass with PWM and filter automation

### Harmonic Patterns (`harmonics/`)

Neo-Riemannian harmonic progressions using P (parallel), R (relative), and L (leading-tone) transformations:

- **Transformation Sequences**: Specific P, R, L sequences for different moods
- **Emotional Mapping**: Where tension and release occur in the progression
- **Voicing Suggestions**: How to voice chords for different electronic styles
- **Arrangement Integration**: How progressions work with different tempos and styles

#### Key Progressions:
- `classic_plr.toml` - Fundamental P-L-R transformations
- `drone_sustained.toml` - Minimal harmonic movement with sustained tones
- `trance_epic_progression.toml` - Emotional trance progressions with peaks and valleys
- `ambient_flow.totml` - Gentle, flowing progressions for ambient styles

## Using the Library

### In the TUI

1. **Navigate Categories**: Use Tab to move between Drums, Bass, Harmony, Melody, etc.
2. **Select Patterns**: Use arrow keys and Enter to choose patterns
3. **Preview**: Press Space to generate a short preview of current selections
4. **Generate**: Press 'G' to create a full composition

### Pattern Combination Guidelines

**For Minimal Techno Berlin Style:**
- Drums: `minimal_techno.toml`  
- Bass: `minimal_techno_bass.toml`
- Harmony: `drone_sustained.toml`
- Melody Strategy: `textural_pads`

**For Ambient Techno Detroit Style:**
- Drums: `ambient_techno.toml`
- Bass: `ambient_organic_bass.toml`  
- Harmony: `ambient_flow.toml`
- Melody Strategy: `lead_synth`

**For Dub Techno Style:**
- Drums: `dub_techno_deep.toml`
- Bass: `dub_techno_deep_bass.toml`
- Harmony: `drone_gradual_shift.toml`
- Melody Strategy: `textural_pads`

## Sound Design Philosophy

The library follows these core principles:

### 1. **Historical Accuracy**
- Patterns and voicing suggestions based on classic tracks and producers
- Hardware and software references from the actual scenes and eras
- Production techniques that capture the essence of each style

### 2. **Practical Guidance**  
- Specific synthesis parameters rather than vague descriptions
- Frequency ranges, envelope settings, and effect parameters
- Mixing and arrangement advice for each style

### 3. **Creative Flexibility**
- Templates provide starting points, not rigid rules
- Mix and match elements from different styles for hybrid approaches
- Extensive parameter suggestions allow for personalization

### 4. **Electronic Music Focus**
- All guidance tailored for electronic music production
- Emphasis on synthesizer programming and electronic effects
- Consideration for club sound systems and electronic music contexts

## Expanding the Library

To add new patterns or styles:

1. **Study the existing templates** to understand the format and level of detail
2. **Research the style thoroughly** - listen to classic tracks, study production techniques
3. **Include comprehensive voicing suggestions** - specific synthesis parameters and mixing advice
4. **Test combinations** to ensure patterns work well together
5. **Document historical context** - artists, labels, hardware, and cultural background

The goal is to provide not just rhythmic patterns, but complete production guidance that helps users create authentic-sounding electronic music in any style.