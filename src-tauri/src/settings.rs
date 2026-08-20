use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::jsonio::{read_fresh_json, write_json};

pub const DEFAULT_THEME: &str = "dark";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub theme: String,
    pub sources: SourceSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: DEFAULT_THEME.to_string(),
            sources: SourceSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SourceSettings {
    pub claude_code: bool,
    pub pi: bool,
    pub opencode: bool,
    pub codex: bool,
    pub gemini: bool,
    pub copilot: bool,
}

impl Default for SourceSettings {
    fn default() -> Self {
        Self {
            claude_code: true,
            pi: true,
            opencode: true,
            codex: true,
            gemini: false,
            copilot: false,
        }
    }
}

impl SourceSettings {
    pub fn enabled(&self, id: &str) -> bool {
        match id {
            "claudeCode" => self.claude_code,
            "pi" => self.pi,
            "opencode" => self.opencode,
            "codex" => self.codex,
            "gemini" => self.gemini,
            "copilot" => self.copilot,
            _ => false,
        }
    }
}

pub fn settings_file() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    base.join("tokenmaxing").join("settings.json")
}

pub fn load_settings() -> Settings {
    read_fresh_json::<Settings>(&settings_file(), f64::INFINITY).unwrap_or_default()
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    write_json(&settings_file(), settings).map_err(|failure| failure.to_string())
}
