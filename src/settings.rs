use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OverlaySettings {
    pub font_size: i32,
    pub color: String,
    pub claude_prefix: String,
    pub codex_prefix: String,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            font_size: 13,
            color: "#FFFFFF".to_string(),
            claude_prefix: "C".to_string(),
            codex_prefix: "X".to_string(),
        }
    }
}

impl OverlaySettings {
    pub fn load() -> Self {
        let path = settings_path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            let settings = Self::default();
            let _ = settings.save();
            return settings;
        };
        toml::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn text_color_bgr(&self) -> u32 {
        parse_hex_rgb(&self.color)
            .map(|(r, g, b)| ((b as u32) << 16) | ((g as u32) << 8) | r as u32)
            .unwrap_or(0x00ffffff)
    }
}

pub fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("cc-balance-overlay")
        .join("settings.toml")
}

fn parse_hex_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let raw = value.trim().trim_start_matches('#');
    if raw.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&raw[0..2], 16).ok()?;
    let g = u8::from_str_radix(&raw[2..4], 16).ok()?;
    let b = u8::from_str_radix(&raw[4..6], 16).ok()?;
    Some((r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_color_bgr_preserves_rgb_channels_for_colorref() {
        let red = OverlaySettings {
            color: "#FF0000".to_string(),
            ..Default::default()
        };
        let blue = OverlaySettings {
            color: "#0000FF".to_string(),
            ..Default::default()
        };

        assert_eq!(red.text_color_bgr(), 0x000000ff);
        assert_eq!(blue.text_color_bgr(), 0x00ff0000);
    }
}
