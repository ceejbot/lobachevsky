//! MIDI file generation and output

use std::path::Path;

use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};

use crate::LobachevskyError;
use crate::core::{Chord, Note};
use crate::rhythm::DrumEvent;

/// MIDI file builder
pub struct MidiFile {
    tracks: Vec<Track<'static>>,
    ticks_per_beat: u16,
    tempo: u32, // microseconds per beat
}

impl MidiFile {
    /// Create a new MIDI file with specified tempo (BPM)
    pub fn new(bpm: u16) -> Self {
        let tempo = 60_000_000 / bpm as u32; // Convert BPM to microseconds per beat
        MidiFile {
            tracks: Vec::new(),
            ticks_per_beat: 480, // Standard resolution
            tempo,
        }
    }

    /// Add a new track
    pub fn add_track(&mut self) -> TrackBuilder {
        TrackBuilder::new(self.ticks_per_beat, self.tempo)
    }

    /// Add a completed track
    pub fn add_completed_track(&mut self, track: Track<'static>) {
        self.tracks.push(track);
    }

    /// Write to file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), LobachevskyError> {
        let header = Header::new(Format::Parallel, Timing::Metrical(self.ticks_per_beat.into()));

        let smf = Smf {
            header,
            tracks: self.tracks.clone(),
        };

        smf.save(path)?;
        Ok(())
    }
}

/// Builder for individual MIDI tracks
pub struct TrackBuilder {
    events: Vec<TrackEvent<'static>>,
    ticks_per_beat: u16,
    tempo: u32,
    current_tick: u32,
}

impl TrackBuilder {
    fn new(ticks_per_beat: u16, tempo: u32) -> Self {
        let mut builder = TrackBuilder {
            events: Vec::new(),
            ticks_per_beat,
            tempo,
            current_tick: 0,
        };

        // Add tempo meta event at the beginning
        builder.add_tempo();
        builder
    }

    fn add_tempo(&mut self) {
        self.events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(self.tempo.into())),
        });
    }

    /// Add a note with duration
    pub fn add_note(&mut self, note: Note, start_beat: f64, duration_beats: f64, velocity: u8, channel: u8) {
        let start_tick = (start_beat * self.ticks_per_beat as f64) as u32;
        let end_tick = ((start_beat + duration_beats) * self.ticks_per_beat as f64) as u32;

        // Note On
        let delta_on = start_tick.saturating_sub(self.current_tick);
        self.events.push(TrackEvent {
            delta: delta_on.into(),
            kind: TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::NoteOn {
                    key: note.to_midi().into(),
                    vel: velocity.into(),
                },
            },
        });
        self.current_tick = start_tick;

        // Note Off
        let delta_off = end_tick.saturating_sub(self.current_tick);
        self.events.push(TrackEvent {
            delta: delta_off.into(),
            kind: TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::NoteOff {
                    key: note.to_midi().into(),
                    vel: 0.into(),
                },
            },
        });
        self.current_tick = end_tick;
    }

    /// Add a chord (all notes start and end together)
    pub fn add_chord(
        &mut self,
        chord: Chord,
        octave: i8,
        start_beat: f64,
        duration_beats: f64,
        velocity: u8,
        channel: u8,
    ) {
        let notes = chord.notes(octave);
        let start_tick = (start_beat * self.ticks_per_beat as f64) as u32;
        let end_tick = ((start_beat + duration_beats) * self.ticks_per_beat as f64) as u32;

        // Add all Note On events
        for (i, note) in notes.iter().enumerate() {
            let delta = if i == 0 {
                start_tick.saturating_sub(self.current_tick)
            } else {
                0
            };

            self.events.push(TrackEvent {
                delta: delta.into(),
                kind: TrackEventKind::Midi {
                    channel: channel.into(),
                    message: MidiMessage::NoteOn {
                        key: note.to_midi().into(),
                        vel: velocity.into(),
                    },
                },
            });

            if i == 0 {
                self.current_tick = start_tick;
            }
        }

        // Add all Note Off events
        for (i, note) in notes.iter().enumerate() {
            let delta = if i == 0 {
                end_tick.saturating_sub(self.current_tick)
            } else {
                0
            };

            self.events.push(TrackEvent {
                delta: delta.into(),
                kind: TrackEventKind::Midi {
                    channel: channel.into(),
                    message: MidiMessage::NoteOff {
                        key: note.to_midi().into(),
                        vel: 0.into(),
                    },
                },
            });

            if i == 0 {
                self.current_tick = end_tick;
            }
        }
    }

    /// Add a drum hit (for rhythm track)
    pub fn add_drum_hit(&mut self, event: &DrumEvent, bar: usize) {
        let beat_position = bar as f64 * 4.0 + event.beat.0;
        let start_tick = (beat_position * self.ticks_per_beat as f64) as u32;

        // Drum channel is typically 9 (0-indexed)
        const DRUM_CHANNEL: u8 = 9;

        // Note On
        let delta_on = start_tick.saturating_sub(self.current_tick);
        self.events.push(TrackEvent {
            delta: delta_on.into(),
            kind: TrackEventKind::Midi {
                channel: DRUM_CHANNEL.into(),
                message: MidiMessage::NoteOn {
                    key: event.voice.midi_note().into(),
                    vel: event.velocity.into(),
                },
            },
        });
        self.current_tick = start_tick;

        // Immediate Note Off for drums
        self.events.push(TrackEvent {
            delta: 10.into(), // Very short duration
            kind: TrackEventKind::Midi {
                channel: DRUM_CHANNEL.into(),
                message: MidiMessage::NoteOff {
                    key: event.voice.midi_note().into(),
                    vel: 0.into(),
                },
            },
        });
        self.current_tick += 10;
    }

    /// Add a chord progression
    pub fn add_progression(&mut self, chords: &[Chord], octave: i8, bars_per_chord: usize, velocity: u8, channel: u8) {
        for (i, chord) in chords.iter().enumerate() {
            let start_beat = (i * bars_per_chord * 4) as f64;
            let duration = (bars_per_chord * 4) as f64;
            self.add_chord(*chord, octave, start_beat, duration, velocity, channel);
        }
    }

    /// Build the track
    pub fn build(mut self) -> Track<'static> {
        // Add end of track event
        self.events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        self.events
    }
}

/// High-level MIDI composition builder
pub struct Composition {
    midi_file: MidiFile,
}

impl Composition {
    pub fn new(bpm: u16) -> Self {
        Composition {
            midi_file: MidiFile::new(bpm),
        }
    }

    /// Add a harmony track with chord progression
    pub fn add_harmony_track(&mut self, chords: &[Chord], octave: i8, bars_per_chord: usize) -> &mut Self {
        let mut track = self.midi_file.add_track();
        track.add_progression(chords, octave, bars_per_chord, 70, 0);
        self.midi_file.add_completed_track(track.build());
        self
    }

    /// Add a rhythm track
    pub fn add_rhythm_track(&mut self, pattern: &dyn crate::rhythm::RhythmPattern, num_bars: usize) -> &mut Self {
        let mut track = self.midi_file.add_track();

        for bar in 0..num_bars {
            let events = pattern.events_for_bar(bar);
            for event in events {
                track.add_drum_hit(&event, bar);
            }
        }

        self.midi_file.add_completed_track(track.build());
        self
    }

    /// Add a melody track
    pub fn add_melody_track(&mut self, notes: &[(Note, f64, f64)], channel: u8) -> &mut Self {
        let mut track = self.midi_file.add_track();

        for (note, start_beat, duration) in notes {
            track.add_note(*note, *start_beat, *duration, 65, channel);
        }

        self.midi_file.add_completed_track(track.build());
        self
    }

    /// Add a bass track with appropriate velocity and channel
    pub fn add_bass_track(&mut self, notes: &[(Note, f64, f64)], channel: u8) -> &mut Self {
        let mut track = self.midi_file.add_track();

        for (note, start_beat, duration) in notes {
            // Bass notes are typically louder and deeper
            track.add_note(*note, *start_beat, *duration, 85, channel);
        }

        self.midi_file.add_completed_track(track.build());
        self
    }

    /// Add a rhythm track from drum events
    pub fn add_rhythm_track_from_events(&mut self, events: &[DrumEvent]) -> &mut Self {
        let mut track = self.midi_file.add_track();

        for event in events {
            // Calculate which bar this event falls in
            let bar = (event.beat.0 / 4.0) as usize;
            // Create a new DrumEvent with beat position relative to the bar
            let bar_relative_event = DrumEvent {
                voice: event.voice,
                beat: crate::rhythm::Beat(event.beat.0 % 4.0),
                velocity: event.velocity,
            };
            track.add_drum_hit(&bar_relative_event, bar);
        }

        self.midi_file.add_completed_track(track.build());
        self
    }

    /// Save the composition to a MIDI file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), LobachevskyError> {
        self.midi_file.save(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ChordQuality, PitchClass};

    #[test]
    fn can_write_midi_files() {
        let mut midi = MidiFile::new(120);
        let mut track = midi.add_track();

        let c_major = Chord::new(PitchClass::C, ChordQuality::Major);
        track.add_chord(c_major, 4, 0.0, 4.0, 64, 0);

        midi.add_completed_track(track.build());

        // Should have one track
        assert_eq!(midi.tracks.len(), 1);
    }
}
