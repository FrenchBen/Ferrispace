//! Configuration types and loading logic for RustSpace.
//!
//! The TOML shape mirrors AeroSpace's config format closely so that users
//! familiar with AeroSpace can adapt their configs with minimal changes.
//!
//! Config resolution order (last writer wins):
//!   1. Built-in defaults (compiled-in via `include_str!`)
//!   2. User config at `~/.config/rustspace/config.toml` (merged field-by-field)

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

/// The fully-resolved runtime config. All fields carry concrete values; no
/// `Option` wrappers survive past the merge step.
#[derive(Debug, Clone, Serialize)]
pub struct Config {
    /// Schema version — matches AeroSpace's `config-version` key.
    pub config_version: u32,

    /// Commands to run once after the daemon starts.
    /// Mirrors AeroSpace `after-startup-command`.
    pub after_startup_command: Vec<String>,

    /// Pixel gap between adjacent windows (inner) and between windows and
    /// monitor edges (outer). Mirrors AeroSpace `gaps.*`.
    pub gaps: Gaps,

    /// Virtual workspace names that stay alive even when empty.
    /// Mirrors AeroSpace `persistent-workspaces`.
    pub persistent_workspaces: Vec<String>,

    /// Whether focus warps to the window under the cursor when it moves.
    /// Mirrors AeroSpace `focus-follows-mouse.enabled`.
    pub focus_follows_mouse: FocusFollowsMouse,

    /// Keyboard preset (`qwerty`, `dvorak`, `colemak`).
    /// Mirrors AeroSpace `key-mapping.preset`.
    pub key_mapping: KeyMapping,

    /// Pixel padding for accordion-layout containers.
    /// Mirrors AeroSpace `accordion-padding`.
    pub accordion_padding: u32,

    /// Default layout for the root container of each workspace.
    /// Mirrors AeroSpace `default-root-container-layout`.
    pub default_root_container_layout: ContainerLayout,

    /// Default split orientation for the root container.
    /// Mirrors AeroSpace `default-root-container-orientation`.
    pub default_root_container_orientation: ContainerOrientation,

    /// Named binding modes, each with a map of `key → command-string`.
    /// Mirrors AeroSpace `[mode.<name>.binding]`.
    /// The `main` mode is mandatory and is always present after loading.
    pub modes: HashMap<String, ModeConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let mut modes = HashMap::new();
        modes.insert("main".to_string(), ModeConfig::default());

        Config {
            config_version: 2,
            after_startup_command: vec![],
            gaps: Gaps::default(),
            persistent_workspaces: vec![],
            focus_follows_mouse: FocusFollowsMouse::default(),
            key_mapping: KeyMapping::default(),
            accordion_padding: 30,
            default_root_container_layout: ContainerLayout::Tiles,
            default_root_container_orientation: ContainerOrientation::Auto,
            modes,
        }
    }
}

// ---------------------------------------------------------------------------
// Gaps
// ---------------------------------------------------------------------------

/// Pixel spacing configuration.  Mirrors AeroSpace `[gaps]` table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Gaps {
    #[serde(default)]
    pub inner: GapsInner,
    #[serde(default)]
    pub outer: GapsOuter,
}

/// Inner gaps (between adjacent windows).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapsInner {
    /// Horizontal gap between side-by-side windows.
    #[serde(default)]
    pub horizontal: u32,
    /// Vertical gap between stacked windows.
    #[serde(default)]
    pub vertical: u32,
}

impl Default for GapsInner {
    fn default() -> Self {
        GapsInner { horizontal: 0, vertical: 0 }
    }
}

/// Outer gaps (between windows and monitor edges).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapsOuter {
    #[serde(default)]
    pub left: u32,
    #[serde(default)]
    pub right: u32,
    #[serde(default)]
    pub top: u32,
    #[serde(default)]
    pub bottom: u32,
}

impl Default for GapsOuter {
    fn default() -> Self {
        GapsOuter { left: 0, right: 0, top: 0, bottom: 0 }
    }
}

// ---------------------------------------------------------------------------
// Focus follows mouse
// ---------------------------------------------------------------------------

/// Mirrors AeroSpace `focus-follows-mouse` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusFollowsMouse {
    #[serde(default)]
    pub enabled: bool,
}

impl Default for FocusFollowsMouse {
    fn default() -> Self {
        FocusFollowsMouse { enabled: false }
    }
}

// ---------------------------------------------------------------------------
// Key mapping
// ---------------------------------------------------------------------------

/// Mirrors AeroSpace `key-mapping` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMapping {
    /// Keyboard layout preset. Mirrors AeroSpace `key-mapping.preset`.
    #[serde(default)]
    pub preset: KeyMappingPreset,
}

impl Default for KeyMapping {
    fn default() -> Self {
        KeyMapping { preset: KeyMappingPreset::Qwerty }
    }
}

/// Keyboard layout presets supported by RustSpace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyMappingPreset {
    Qwerty,
    Dvorak,
    Colemak,
}

impl Default for KeyMappingPreset {
    fn default() -> Self {
        KeyMappingPreset::Qwerty
    }
}

impl fmt::Display for KeyMappingPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyMappingPreset::Qwerty => write!(f, "qwerty"),
            KeyMappingPreset::Dvorak => write!(f, "dvorak"),
            KeyMappingPreset::Colemak => write!(f, "colemak"),
        }
    }
}

// ---------------------------------------------------------------------------
// Container layout / orientation
// ---------------------------------------------------------------------------

/// Layout algorithm for container nodes.
/// Mirrors AeroSpace `default-root-container-layout`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerLayout {
    Tiles,
    Accordion,
}

impl Default for ContainerLayout {
    fn default() -> Self {
        ContainerLayout::Tiles
    }
}

impl fmt::Display for ContainerLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerLayout::Tiles => write!(f, "tiles"),
            ContainerLayout::Accordion => write!(f, "accordion"),
        }
    }
}

/// Split axis for container nodes.
/// Mirrors AeroSpace `default-root-container-orientation`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerOrientation {
    /// Always split left-right.
    Horizontal,
    /// Always split top-bottom.
    Vertical,
    /// Wide monitor → horizontal; tall monitor → vertical.
    Auto,
}

impl Default for ContainerOrientation {
    fn default() -> Self {
        ContainerOrientation::Auto
    }
}

impl fmt::Display for ContainerOrientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerOrientation::Horizontal => write!(f, "horizontal"),
            ContainerOrientation::Vertical => write!(f, "vertical"),
            ContainerOrientation::Auto => write!(f, "auto"),
        }
    }
}

// ---------------------------------------------------------------------------
// Binding modes
// ---------------------------------------------------------------------------

/// A named binding mode with its key→command table.
/// Mirrors AeroSpace `[mode.<name>.binding]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModeConfig {
    /// Maps key-chord strings (e.g. `"alt-h"`) to command strings
    /// (e.g. `"focus left"`).
    #[serde(default)]
    pub binding: HashMap<String, BindingValue>,
}

/// A binding value: either a single command string or a sequence of commands.
///
/// AeroSpace allows both:
///   `alt-h = 'focus left'`                 — single
///   `esc = ['reload-config', 'mode main']`  — sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BindingValue {
    Single(String),
    Sequence(Vec<String>),
}

impl BindingValue {
    /// Return the commands as a slice regardless of variant.
    pub fn commands(&self) -> &[String] {
        match self {
            BindingValue::Single(s) => std::slice::from_ref(s),
            BindingValue::Sequence(v) => v.as_slice(),
        }
    }
}

// ---------------------------------------------------------------------------
// Raw TOML deserialization target (all fields optional for merging)
// ---------------------------------------------------------------------------

/// Intermediate struct that serde deserializes user TOML into.
/// Every field is `Option<T>` so absent keys simply leave the resolved
/// config's defaults untouched during the merge step.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RawConfig {
    pub config_version: Option<u32>,
    pub after_startup_command: Option<Vec<String>>,
    pub gaps: Option<Gaps>,
    pub persistent_workspaces: Option<Vec<String>>,
    #[serde(rename = "focus-follows-mouse")]
    pub focus_follows_mouse: Option<FocusFollowsMouse>,
    #[serde(rename = "key-mapping")]
    pub key_mapping: Option<KeyMapping>,
    pub accordion_padding: Option<u32>,
    pub default_root_container_layout: Option<ContainerLayout>,
    pub default_root_container_orientation: Option<ContainerOrientation>,
    /// `[mode.<name>]` tables.
    pub mode: Option<HashMap<String, RawModeOuter>>,
}

/// Wrapper for the two-level `[mode.<name>.binding]` TOML structure.
///
/// ```toml
/// [mode.main.binding]
/// alt-h = 'focus left'
/// ```
#[derive(Debug, Default, Deserialize)]
pub struct RawModeOuter {
    #[serde(default)]
    pub binding: HashMap<String, BindingValue>,
}

// ---------------------------------------------------------------------------
// Config loading
// ---------------------------------------------------------------------------

/// Errors that can occur while loading configuration.
#[derive(Debug)]
pub enum ConfigError {
    TomlParse(toml::de::Error),
    Io(std::io::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::TomlParse(e) => write!(f, "TOML parse error: {e}"),
            ConfigError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::TomlParse(e)
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

/// Parse a TOML string into a `RawConfig`.
fn parse_raw(source: &str) -> Result<RawConfig, ConfigError> {
    toml::from_str(source).map_err(ConfigError::from)
}

/// Merge a `RawConfig` on top of a `Config` baseline.
/// Only fields that are `Some(…)` in `raw` override the baseline.
fn merge(mut base: Config, raw: RawConfig) -> Config {
    if let Some(v) = raw.config_version {
        base.config_version = v;
    }
    if let Some(v) = raw.after_startup_command {
        base.after_startup_command = v;
    }
    if let Some(v) = raw.gaps {
        base.gaps = v;
    }
    if let Some(v) = raw.persistent_workspaces {
        base.persistent_workspaces = v;
    }
    if let Some(v) = raw.focus_follows_mouse {
        base.focus_follows_mouse = v;
    }
    if let Some(v) = raw.key_mapping {
        base.key_mapping = v;
    }
    if let Some(v) = raw.accordion_padding {
        base.accordion_padding = v;
    }
    if let Some(v) = raw.default_root_container_layout {
        base.default_root_container_layout = v;
    }
    if let Some(v) = raw.default_root_container_orientation {
        base.default_root_container_orientation = v;
    }

    // Merge modes: user bindings are applied key-by-key within each mode.
    // A user-defined mode's bindings are overlaid on top of the baseline
    // mode's bindings — only keys present in the user config are overwritten;
    // all other baseline bindings in that mode are preserved.
    // A brand-new mode name (not in baseline) is inserted wholesale.
    if let Some(raw_modes) = raw.mode {
        for (name, outer) in raw_modes {
            let mode_entry = base.modes.entry(name).or_insert_with(ModeConfig::default);
            for (key, val) in outer.binding {
                mode_entry.binding.insert(key, val);
            }
        }
    }

    base
}

/// Load the default config (compiled in via `include_str!`) and optionally
/// merge the user config from `~/.config/rustspace/config.toml`.
///
/// Returns the fully-resolved `Config` and a list of non-fatal warnings.
pub fn load_config() -> Result<(Config, Vec<String>), ConfigError> {
    let default_toml = include_str!("../resources/default-config.toml");
    let raw_default = parse_raw(default_toml)?;

    // Start from the hard-coded Rust defaults, then layer the built-in TOML.
    let base = merge(Config::default(), raw_default);

    let mut warnings: Vec<String> = vec![];

    // Attempt to load the user config; it is optional.
    // Honour XDG_CONFIG_HOME if set, otherwise fall back to ~/.config.
    let user_config_path = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME").ok().map(|h| {
                std::path::PathBuf::from(h).join(".config")
            })
        })
        .map(|cfg_dir| cfg_dir.join("rustspace").join("config.toml"));

    let resolved = match user_config_path {
        Some(path) if path.exists() => {
            let src = std::fs::read_to_string(&path)?;
            match parse_raw(&src) {
                Ok(raw_user) => merge(base, raw_user),
                Err(e) => {
                    warnings.push(format!(
                        "Could not parse user config at {}: {}. Using defaults.",
                        path.display(),
                        e
                    ));
                    base
                }
            }
        }
        Some(_) => {
            // Path resolved but file does not exist — silently use defaults.
            base
        }
        None => {
            warnings.push(
                "Could not determine user config directory. Using defaults.".to_string(),
            );
            base
        }
    };

    Ok((resolved, warnings))
}

// ---------------------------------------------------------------------------
// Test helpers — expose internal functions for integration tests in main.rs
// ---------------------------------------------------------------------------

#[cfg(test)]
pub fn parse_raw_for_test(source: &str) -> Result<RawConfig, ConfigError> {
    parse_raw(source)
}

#[cfg(test)]
pub fn merge_for_test(base: Config, raw: RawConfig) -> Config {
    merge(base, raw)
}
