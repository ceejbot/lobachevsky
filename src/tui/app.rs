//! Main TUI application state and logic

use super::browser::*;
use super::preview::*;
use crate::LobachevskyError;
use crate::bass::BassStrategy;
use crate::generators::algorithmic::GenerateInput;
use crate::harmony::TypedHarmonicPattern;
use crate::library::Library;
use crate::melody::MelodyStrategy;

/// Main TUI application state
pub struct App {
    /// Current active category/mode
    pub mode: AppMode,
    /// Whether the app should exit
    pub should_exit: bool,
    /// Current selections for each category
    pub selections: Selections,
    /// File browsers for different categories
    pub browsers: Browsers,
    /// Preview information
    pub preview: PreviewInfo,
    /// Status messages
    pub status: String,
    /// Current parameter being edited (for Params mode)
    pub param_editor: ParamEditor,
}

/// Different modes/categories in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Drums,
    Bass,
    Harmony,
    Melody,
    Groove,
    Params,
    Preview,
}

/// User selections across all categories
#[derive(Debug, Clone)]
pub struct Selections {
    pub drums_pattern: Option<String>,
    pub bass_pattern: Option<String>,
    pub harmony_pattern: Option<String>,
    pub melody_strategy: Option<String>,
    pub groove_settings: Option<String>, // For humanization options
    pub bass_strategy: Option<String>,   // Bass generation strategy
    pub bars: usize,
    pub tempo: Option<u16>,
    pub output_file: String,
    pub key_signature: String,  // Musical key (e.g., "C", "Dm", "F#")
    pub starting_chord: String, // Starting chord (e.g., "Cmaj", "Am", "F#m")
}

/// File browsers for different content types
pub struct Browsers {
    pub drums_browser: FileBrowser,
    pub bass_browser: FileBrowser,
    pub harmony_browser: FileBrowser,
    pub melody_browser: MelodyBrowser,
    pub groove_browser: GrooveBrowser,
}

/// Available melody strategies
#[derive(Debug, Clone)]
pub struct MelodyBrowser {
    pub strategies: Vec<String>,
    pub selected_index: usize,
}

/// Available groove/humanization options
#[derive(Debug, Clone)]
pub struct GrooveBrowser {
    pub groove_types: Vec<String>,
    pub selected_index: usize,
}

/// Parameter editor state
#[derive(Debug, Clone)]
pub struct ParamEditor {
    pub selected_param: usize,
    pub param_names: Vec<String>,
    pub editing_mode: bool,
    pub edit_buffer: String,
}

#[derive(Debug, Clone, Copy)]
pub enum EditableParam {
    Bars,
    Tempo,
    OutputFile,
    KeySignature,
    StartingChord,
    BassStrategy,
}

impl Default for Selections {
    fn default() -> Self {
        Selections {
            drums_pattern: None,
            bass_pattern: None,
            harmony_pattern: None,
            melody_strategy: Some("mixed".to_string()),
            groove_settings: Some("human".to_string()),
            bass_strategy: Some("root".to_string()),
            bars: 64,
            tempo: None,
            output_file: "generated.mid".to_string(),
            key_signature: "C".to_string(),
            starting_chord: "Cmaj".to_string(),
        }
    }
}

impl MelodyBrowser {
    pub fn new() -> Self {
        MelodyBrowser {
            strategies: vec![
                "lead_synth".to_string(),
                "arpeggiated".to_string(),
                "rhythmic_stabs".to_string(),
                "textural_pads".to_string(),
                "pluck_sequence".to_string(),
                "bass_lead".to_string(),
            ],
            selected_index: 0,
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.strategies.get(self.selected_index).map(|s| s.as_str())
    }

    pub fn next(&mut self) {
        if self.selected_index < self.strategies.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
}

impl Default for MelodyBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GrooveBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ParamEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl GrooveBrowser {
    pub fn new() -> Self {
        GrooveBrowser {
            groove_types: vec![
                "human".to_string(),
                "tight".to_string(),
                "loose".to_string(),
                "swing".to_string(),
                "shuffle".to_string(),
            ],
            selected_index: 0,
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.groove_types.get(self.selected_index).map(|s| s.as_str())
    }

    pub fn next(&mut self) {
        if self.selected_index < self.groove_types.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
}

impl ParamEditor {
    pub fn new() -> Self {
        ParamEditor {
            selected_param: 0,
            param_names: vec![
                "Bars".to_string(),
                "Tempo".to_string(),
                "Output File".to_string(),
                "Key Signature".to_string(),
                "Starting Chord".to_string(),
                "Bass Strategy".to_string(),
            ],
            editing_mode: false,
            edit_buffer: String::new(),
        }
    }

    pub fn next(&mut self) {
        if self.selected_param < self.param_names.len().saturating_sub(1) {
            self.selected_param += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected_param > 0 {
            self.selected_param -= 1;
        }
    }

    pub fn start_editing(&mut self, current_value: &str) {
        self.editing_mode = true;
        self.edit_buffer = current_value.to_string();
    }

    pub fn stop_editing(&mut self) {
        self.editing_mode = false;
        self.edit_buffer.clear();
    }

    pub fn add_char(&mut self, c: char) {
        if self.editing_mode {
            self.edit_buffer.push(c);
        }
    }

    pub fn backspace(&mut self) {
        if self.editing_mode {
            self.edit_buffer.pop();
        }
    }

    pub fn current_param(&self) -> EditableParam {
        match self.selected_param {
            0 => EditableParam::Bars,
            1 => EditableParam::Tempo,
            2 => EditableParam::OutputFile,
            3 => EditableParam::KeySignature,
            4 => EditableParam::StartingChord,
            5 => EditableParam::BassStrategy,
            _ => EditableParam::Bars,
        }
    }
}

impl App {
    pub fn new() -> Result<Self, LobachevskyError> {
        let drums_browser = FileBrowser::new("library/patterns", "drum patterns")?;
        let bass_browser = FileBrowser::new("library/bass", "bass patterns")?;
        let harmony_browser = FileBrowser::new("library/harmonics", "harmony patterns")?;
        let melody_browser = MelodyBrowser::new();
        let groove_browser = GrooveBrowser::new();

        Ok(App {
            mode: AppMode::Drums,
            should_exit: false,
            selections: Selections::default(),
            browsers: Browsers {
                drums_browser,
                bass_browser,
                harmony_browser,
                melody_browser,
                groove_browser,
            },
            preview: PreviewInfo::default(),
            status: "Ready to create music! Use Tab to navigate categories.".to_string(),
            param_editor: ParamEditor::new(),
        })
    }

    /// Move to the next category
    pub fn next_mode(&mut self) {
        self.mode = match self.mode {
            AppMode::Drums => AppMode::Bass,
            AppMode::Bass => AppMode::Harmony,
            AppMode::Harmony => AppMode::Melody,
            AppMode::Melody => AppMode::Groove,
            AppMode::Groove => AppMode::Params,
            AppMode::Params => AppMode::Preview,
            AppMode::Preview => AppMode::Drums,
        };
    }

    /// Move to the previous category
    pub fn previous_mode(&mut self) {
        self.mode = match self.mode {
            AppMode::Drums => AppMode::Preview,
            AppMode::Bass => AppMode::Drums,
            AppMode::Harmony => AppMode::Bass,
            AppMode::Melody => AppMode::Harmony,
            AppMode::Groove => AppMode::Melody,
            AppMode::Params => AppMode::Groove,
            AppMode::Preview => AppMode::Params,
        };
    }

    /// Handle navigation within the current mode
    pub fn navigate_up(&mut self) {
        match self.mode {
            AppMode::Drums => self.browsers.drums_browser.previous(),
            AppMode::Bass => self.browsers.bass_browser.previous(),
            AppMode::Harmony => self.browsers.harmony_browser.previous(),
            AppMode::Melody => self.browsers.melody_browser.previous(),
            AppMode::Groove => self.browsers.groove_browser.previous(),
            AppMode::Params => self.param_editor.previous(),
            AppMode::Preview => {}
        }
    }

    pub fn navigate_down(&mut self) {
        match self.mode {
            AppMode::Drums => self.browsers.drums_browser.next(),
            AppMode::Bass => self.browsers.bass_browser.next(),
            AppMode::Harmony => self.browsers.harmony_browser.next(),
            AppMode::Melody => self.browsers.melody_browser.next(),
            AppMode::Groove => self.browsers.groove_browser.next(),
            AppMode::Params => self.param_editor.next(),
            AppMode::Preview => {}
        }
    }

    /// Select the current item
    pub fn select_current(&mut self) -> Result<(), LobachevskyError> {
        match self.mode {
            AppMode::Drums => {
                if let Some(selected) = self.browsers.drums_browser.selected_file() {
                    self.selections.drums_pattern = Some(selected.to_string());
                    self.update_preview()?;
                }
            }
            AppMode::Bass => {
                if let Some(selected) = self.browsers.bass_browser.selected_file() {
                    self.selections.bass_pattern = Some(selected.to_string());
                    self.update_preview()?;
                }
            }
            AppMode::Harmony => {
                if let Some(selected) = self.browsers.harmony_browser.selected_file() {
                    self.selections.harmony_pattern = Some(selected.to_string());
                    self.update_preview()?;
                }
            }
            AppMode::Melody => {
                if let Some(selected) = self.browsers.melody_browser.selected() {
                    self.selections.melody_strategy = Some(selected.to_string());
                    self.update_preview()?;
                }
            }
            AppMode::Groove => {
                if let Some(selected) = self.browsers.groove_browser.selected() {
                    self.selections.groove_settings = Some(selected.to_string());
                    self.update_preview()?;
                }
            }
            AppMode::Params => {
                if self.param_editor.editing_mode {
                    // Apply the edit
                    self.apply_parameter_edit()?;
                    self.param_editor.stop_editing();
                } else {
                    // Start editing current parameter
                    let current_value = self.get_current_param_value();
                    self.param_editor.start_editing(&current_value);
                }
            }
            AppMode::Preview => {}
        }
        Ok(())
    }

    /// Update the preview based on current selections
    pub fn update_preview(&mut self) -> Result<(), LobachevskyError> {
        self.preview = PreviewInfo::calculate(&self.selections)?;
        Ok(())
    }

    /// Generate music with current selections
    pub fn generate(&mut self) -> Result<(), LobachevskyError> {
        use crate::generators::algorithmic::generate_algorithmic;

        // Validate selections - need at least drums/bass or harmony
        if self.selections.drums_pattern.is_none()
            && self.selections.bass_pattern.is_none()
            && self.selections.harmony_pattern.is_none()
        {
            self.status = "❌ Need at least drums, bass, or harmony pattern to generate".to_string();
            return Ok(());
        }

        self.update_preview()?;
        self.status = "🎵 Generating music...".to_string();

        // Create GenerateInput from TUI selections
        let input = self.create_generate_input()?;

        // Call the main generation function
        match generate_algorithmic(
            input,
            self.selections.bars,
            Some(self.preview.tempo),
            &self.selections.output_file,
        ) {
            Ok(()) => {
                self.status = format!(
                    "✅ Generated {} successfully! ({} bars, {} BPM)",
                    self.selections.output_file, self.selections.bars, self.preview.tempo
                );
            }
            Err(e) => {
                self.status = format!("❌ Generation failed: {}", e);
            }
        }

        Ok(())
    }

    /// Generate a short preview of the current selections
    pub fn generate_preview(&mut self) -> Result<(), LobachevskyError> {
        use crate::generators::algorithmic::generate_algorithmic;

        // Validate selections - need at least drums/bass or harmony
        if self.selections.drums_pattern.is_none()
            && self.selections.bass_pattern.is_none()
            && self.selections.harmony_pattern.is_none()
        {
            self.status = "❌ Need at least drums, bass, or harmony pattern to preview".to_string();
            return Ok(());
        }

        self.update_preview()?;
        self.status = "🎧 Generating preview...".to_string();

        // Create GenerateInput from TUI selections
        let input = self.create_generate_input()?;

        // Generate a shorter preview (8-16 bars instead of full length)
        let preview_bars = (self.selections.bars / 4).clamp(8, 16);
        let preview_filename = format!("preview_{}", self.selections.output_file);

        // Call the main generation function with shorter length
        match generate_algorithmic(input, preview_bars, Some(self.preview.tempo), &preview_filename) {
            Ok(()) => {
                self.status = format!(
                    "🎧 Preview generated: {} ({} bars, {} BPM) - Press Space again for new preview",
                    preview_filename, preview_bars, self.preview.tempo
                );
            }
            Err(e) => {
                self.status = format!("❌ Preview failed: {}", e);
            }
        }

        Ok(())
    }

    /// Convert TUI selections into GenerateInput for the algorithmic generator
    fn create_generate_input(&self) -> Result<GenerateInput, LobachevskyError> {
        // Parse melody strategy - now using electronic-focused strategies
        let melody_strategy = match self.selections.melody_strategy.as_deref() {
            Some("lead_synth") => MelodyStrategy::LeadSynth,
            Some("arpeggiated") => MelodyStrategy::Arpeggiated,
            Some("rhythmic_stabs") => MelodyStrategy::RhythmicStabs,
            Some("textural_pads") => MelodyStrategy::TexturalPads,
            Some("pluck_sequence") => MelodyStrategy::PluckSequence,
            Some("bass_lead") => MelodyStrategy::BassLead,
            // Legacy support for old strategies
            Some("mixed") => MelodyStrategy::LeadSynth, // Map to lead synth
            Some("chord_tones") => MelodyStrategy::TexturalPads, // Map to pads
            Some("arpeggio") => MelodyStrategy::Arpeggiated, // Direct map
            Some("stepwise") => MelodyStrategy::PluckSequence, // Map to pluck
            _ => MelodyStrategy::LeadSynth,             // Default to lead synth
        };

        // Create harmony pattern - if none selected, create a simple one
        let harmony = if let Some(ref harmony_name) = self.selections.harmony_pattern {
            self.load_harmony_pattern(harmony_name)?
        } else {
            // Create a default simple neo-Riemannian pattern
            self.create_default_harmony()?
        };

        // Parse bass strategy
        let bass_strategy = self
            .selections
            .bass_strategy
            .as_ref()
            .map(|strategy_str| BassStrategy::from(strategy_str.as_str()));

        Ok(GenerateInput {
            rhythm_name: self
                .selections
                .drums_pattern
                .clone()
                .or(self.selections.bass_pattern.clone()),
            harmony,
            melody_strategy,
            bass_strategy,
            call_response_type: None, // TODO: Add call-response options
            notes_per_chord: 16,      // TODO: Make this configurable
        })
    }

    /// Load harmony pattern from file
    fn load_harmony_pattern(&self, name: &str) -> Result<TypedHarmonicPattern, LobachevskyError> {
        use crate::library::{HARMONICS_LIB, HarmonicLibrary};

        let mut lib = HarmonicLibrary::new();
        lib.load_from_directory(std::path::Path::new(HARMONICS_LIB))?;

        let pattern = lib.get(name).ok_or_else(|| LobachevskyError::ParseError {
            message: format!("Harmony pattern '{}' not found", name),
        })?;

        pattern.to_typed()
    }

    /// Create a default harmony pattern when none is selected
    fn create_default_harmony(&self) -> Result<TypedHarmonicPattern, LobachevskyError> {
        use crate::{Chord, Transform};

        // Parse the user's starting chord selection
        let start_chord = Chord::try_from(self.selections.starting_chord.as_str()).unwrap_or(Chord::c_major()); // Fallback to C major if parsing fails

        // Create a simple TypedHarmonicPattern with basic neo-Riemannian
        // transformations
        Ok(TypedHarmonicPattern {
            name: "Default".to_string(),
            description: Some(format!(
                "Auto-generated progression starting from {}",
                self.selections.starting_chord
            )),
            start_chord,
            transformations: vec![Transform::P, Transform::R, Transform::L],
            mode: None,
            tonic: None,
            return_to_start: true,
            tempo_hint: self.selections.tempo,
            bars_hint: Some(self.selections.bars),
        })
    }

    /// Get the current category name for display
    pub fn mode_name(&self) -> &'static str {
        match self.mode {
            AppMode::Drums => "Drums",
            AppMode::Bass => "Bass",
            AppMode::Harmony => "Harmony",
            AppMode::Melody => "Melody",
            AppMode::Groove => "Groove",
            AppMode::Params => "Params",
            AppMode::Preview => "Preview",
        }
    }

    /// Get current parameter value as string for editing
    fn get_current_param_value(&self) -> String {
        match self.param_editor.current_param() {
            EditableParam::Bars => self.selections.bars.to_string(),
            EditableParam::Tempo => self
                .selections
                .tempo
                .map(|t| t.to_string())
                .unwrap_or_else(|| "Auto".to_string()),
            EditableParam::OutputFile => self.selections.output_file.clone(),
            EditableParam::KeySignature => self.selections.key_signature.clone(),
            EditableParam::StartingChord => self.selections.starting_chord.clone(),
            EditableParam::BassStrategy => self
                .selections
                .bass_strategy
                .clone()
                .unwrap_or_else(|| "root".to_string()),
        }
    }

    /// Apply parameter edit from buffer
    fn apply_parameter_edit(&mut self) -> Result<(), LobachevskyError> {
        match self.param_editor.current_param() {
            EditableParam::Bars => {
                if let Ok(bars) = self.param_editor.edit_buffer.parse::<usize>() {
                    if (16..=512).contains(&bars) {
                        self.selections.bars = bars;
                        self.update_preview()?;
                        self.status = format!("Set bars to {}", bars);
                    } else {
                        self.status = "Bars must be between 16 and 512".to_string();
                    }
                } else {
                    self.status = "Invalid number for bars".to_string();
                }
            }
            EditableParam::Tempo => {
                if self.param_editor.edit_buffer == "Auto" || self.param_editor.edit_buffer.is_empty() {
                    self.selections.tempo = None;
                    self.update_preview()?;
                    self.status = "Set tempo to Auto".to_string();
                } else if let Ok(tempo) = self.param_editor.edit_buffer.parse::<u16>() {
                    if (60..=200).contains(&tempo) {
                        self.selections.tempo = Some(tempo);
                        self.update_preview()?;
                        self.status = format!("Set tempo to {} BPM", tempo);
                    } else {
                        self.status = "Tempo must be between 60 and 200 BPM".to_string();
                    }
                } else {
                    self.status = "Invalid tempo (use number or 'Auto')".to_string();
                }
            }
            EditableParam::OutputFile => {
                if !self.param_editor.edit_buffer.is_empty() {
                    self.selections.output_file = self.param_editor.edit_buffer.clone();
                    self.status = format!("Set output file to {}", self.selections.output_file);
                } else {
                    self.status = "Output file cannot be empty".to_string();
                }
            }
            EditableParam::KeySignature => {
                if !self.param_editor.edit_buffer.is_empty() {
                    // Basic validation - could be enhanced
                    let key = self.param_editor.edit_buffer.trim();
                    if !key.is_empty() && key.len() <= 3 {
                        // e.g., "C", "F#", "Bb"
                        self.selections.key_signature = key.to_string();
                        self.status = format!("Set key signature to {}", key);
                    } else {
                        self.status = "Invalid key signature (e.g., C, F#, Bb)".to_string();
                    }
                } else {
                    self.status = "Key signature cannot be empty".to_string();
                }
            }
            EditableParam::StartingChord => {
                if !self.param_editor.edit_buffer.is_empty() {
                    // Basic validation - could be enhanced
                    let chord = self.param_editor.edit_buffer.trim();
                    if !chord.is_empty() && chord.len() <= 6 {
                        // e.g., "C", "Cmaj", "F#m", "Bbmaj7"
                        self.selections.starting_chord = chord.to_string();
                        self.status = format!("Set starting chord to {}", chord);
                    } else {
                        self.status = "Invalid chord format (e.g., C, Cmaj, F#m)".to_string();
                    }
                } else {
                    self.status = "Starting chord cannot be empty".to_string();
                }
            }
            EditableParam::BassStrategy => {
                if !self.param_editor.edit_buffer.is_empty() {
                    let strategy = self.param_editor.edit_buffer.trim().to_lowercase();
                    let valid_strategies = [
                        "root", "walking", "rhythmic", "counterpoint", "root_fifth", "pedal_c", "pedal_g",
                    ];

                    if valid_strategies.contains(&strategy.as_str()) || strategy.starts_with("pedal_") {
                        self.selections.bass_strategy = Some(strategy.clone());
                        self.status = format!("Set bass strategy to {}", strategy);
                    } else {
                        self.status =
                            "Invalid strategy (root, walking, rhythmic, counterpoint, root_fifth, pedal_C)".to_string();
                    }
                } else {
                    self.status = "Bass strategy cannot be empty".to_string();
                }
            }
        }
        Ok(())
    }

    /// Handle character input in parameter editing mode
    pub fn handle_param_char(&mut self, c: char) {
        if self.param_editor.editing_mode {
            self.param_editor.add_char(c);
        }
    }

    /// Handle backspace in parameter editing mode
    pub fn handle_param_backspace(&mut self) {
        if self.param_editor.editing_mode {
            self.param_editor.backspace();
        }
    }
}
