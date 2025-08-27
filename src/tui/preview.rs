//! Preview and estimation functionality for the TUI

use super::app::Selections;
use crate::LobachevskyError;

/// Information about what will be generated
#[derive(Debug, Clone, Default)]
pub struct PreviewInfo {
    /// Estimated composition length in minutes and seconds
    pub duration: Option<(u32, u32)>,
    /// Total bars
    pub bars: usize,
    /// Final tempo (resolved from hints)
    pub tempo: u16,
    /// Number of tracks that will be generated
    pub track_count: usize,
    /// Brief description of what will be created
    pub description: String,
    /// Any warnings or issues
    pub warnings: Vec<String>,
}

impl PreviewInfo {
    /// Calculate preview information from current selections
    pub fn calculate(selections: &Selections) -> Result<Self, LobachevskyError> {
        let mut preview = PreviewInfo::default();

        preview.bars = selections.bars;

        // Resolve tempo from selections and hints
        preview.tempo = Self::resolve_tempo(selections)?;

        // Calculate duration
        preview.duration = Some(Self::calculate_duration(preview.bars, preview.tempo));

        // Estimate track count
        preview.track_count = Self::estimate_tracks(selections);

        // Generate description
        preview.description = Self::generate_description(selections, &preview);

        // Check for potential issues
        preview.warnings = Self::check_warnings(selections);

        Ok(preview)
    }

    /// Resolve final tempo from selections and pattern hints
    fn resolve_tempo(selections: &Selections) -> Result<u16, LobachevskyError> {
        // If user specified tempo, use that
        if let Some(tempo) = selections.tempo {
            return Ok(tempo);
        }

        // Try to get tempo from drums or bass pattern
        if let Some(pattern_name) = selections.drums_pattern.as_ref().or(selections.bass_pattern.as_ref())
            && let Some(tempo) = Self::get_pattern_tempo_hint(pattern_name)?
        {
            return Ok(tempo);
        }

        // Try to get tempo from harmony pattern
        if let Some(ref harmony_name) = selections.harmony_pattern
            && let Some(tempo) = Self::get_harmony_tempo_hint(harmony_name)?
        {
            return Ok(tempo);
        }

        // Default fallback
        Ok(120)
    }

    /// Get tempo hint from a rhythm pattern file
    fn get_pattern_tempo_hint(pattern_name: &str) -> Result<Option<u16>, LobachevskyError> {
        let path = format!("library/patterns/{}.toml", pattern_name);
        if let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(parsed) = toml::from_str::<toml::Value>(&content)
            && let Some(hint) = parsed.get("tempo_hint")
            && let Some(tempo) = hint.as_integer()
        {
            return Ok(Some(tempo as u16));
        }
        Ok(None)
    }

    /// Get tempo hint from a harmony pattern file
    fn get_harmony_tempo_hint(harmony_name: &str) -> Result<Option<u16>, LobachevskyError> {
        let path = format!("library/harmonics/{}.toml", harmony_name);
        if let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(parsed) = toml::from_str::<toml::Value>(&content)
            && let Some(hint) = parsed.get("tempo_hint")
            && let Some(tempo) = hint.as_integer()
        {
            return Ok(Some(tempo as u16));
        }
        Ok(None)
    }

    /// Calculate duration from bars and tempo
    fn calculate_duration(bars: usize, tempo: u16) -> (u32, u32) {
        // Duration = (bars * 4 beats/bar * 60 seconds/minute) / (tempo beats/minute)
        let total_seconds = (bars * 4 * 60) as f64 / tempo as f64;
        let minutes = (total_seconds / 60.0) as u32;
        let seconds = (total_seconds % 60.0) as u32;
        (minutes, seconds)
    }

    /// Estimate how many tracks will be generated
    fn estimate_tracks(selections: &Selections) -> usize {
        let mut count = 0;

        if selections.drums_pattern.is_some() || selections.bass_pattern.is_some() {
            count += 1; // Rhythm track
        }

        if selections.harmony_pattern.is_some() {
            count += 1; // Harmony track
        }

        if selections.melody_strategy.is_some() {
            count += 1; // Melody track
        }

        count
    }

    /// Generate human-readable description of what will be created
    fn generate_description(selections: &Selections, _preview: &PreviewInfo) -> String {
        let mut parts = Vec::new();

        if let Some(ref drums) = selections.drums_pattern {
            parts.push(format!("Drums: {}", drums));
        }
        if let Some(ref bass) = selections.bass_pattern {
            parts.push(format!("Bass: {}", bass));
        }

        if let Some(ref harmony) = selections.harmony_pattern {
            parts.push(format!("Harmony: {}", harmony));
        }

        if let Some(ref melody) = selections.melody_strategy {
            parts.push(format!("Melody: {}", melody));
        }

        if parts.is_empty() {
            "No selections made yet".to_string()
        } else {
            parts.join(" | ")
        }
    }

    /// Check for potential issues with current selections
    fn check_warnings(selections: &Selections) -> Vec<String> {
        let mut warnings = Vec::new();

        if selections.drums_pattern.is_none() && selections.bass_pattern.is_none() {
            warnings.push("No rhythm pattern selected".to_string());
        }

        if selections.harmony_pattern.is_none() {
            warnings.push("No harmony pattern selected".to_string());
        }

        if selections.bars > 128 {
            warnings.push("Very long composition - consider shorter length".to_string());
        }

        warnings
    }

    /// Format duration for display
    pub fn duration_string(&self) -> String {
        match self.duration {
            Some((minutes, seconds)) => format!("{}:{:02}", minutes, seconds),
            None => "Unknown".to_string(),
        }
    }

    /// Get a summary line for display
    pub fn summary_line(&self) -> String {
        format!(
            "{} bars, {} @ {} BPM, {} tracks",
            self.bars,
            self.duration_string(),
            self.tempo,
            self.track_count
        )
    }

    /// Check if ready to generate
    pub fn is_ready(&self) -> bool {
        self.warnings.is_empty()
    }
}
