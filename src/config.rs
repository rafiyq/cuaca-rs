//! Render/output configuration shared by all render functions.

use serde::{Deserialize, Serialize};

use crate::color::ColorMode;
use crate::lang::Lang;

/// Choose what a renderer normally produces end-to-end (informational hint for
/// consumers; both render functions can be called explicitly instead).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Waybar JSON (default)
    #[default]
    Bar,
    /// Plain terminal display
    Text,
    /// Raw forecast JSON
    Json,
}

/// Configuration for the directed render functions. Mirrors the old CLI's
/// defaults; serializes compactly for daemon wire protocols.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub lang: Lang,
    pub format: OutputFormat,
    /// `{KEY}` expression for the bar display (`{ICON}` accepted as well).
    pub custom_indicator: Option<String>,
    /// strftime format used for localized date headers.
    pub date_format: String,
    pub ampm: bool,
    /// Use nerd font symbols instead of emojis.
    pub nerd: bool,
    /// Hide cloud cover / precipitation / visibility per-slot details.
    pub hide_details: bool,
    /// Display temperature in Fahrenheit instead of Celsius.
    pub fahrenheit: bool,
    /// Number of Y-axis tick labels per panel (3-6 effective range).
    pub yticks: usize,
    pub color: ColorMode,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            lang: Lang::default(),
            format: OutputFormat::default(),
            custom_indicator: None,
            date_format: "%Y-%m-%d".to_string(),
            ampm: false,
            nerd: false,
            hide_details: false,
            fahrenheit: false,
            yticks: 4,
            color: ColorMode::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_old_cli() {
        let c = Config::default();
        assert_eq!(c.lang, Lang::EN);
        assert_eq!(c.format, OutputFormat::Bar);
        assert_eq!(c.date_format, "%Y-%m-%d");
        assert_eq!(c.yticks, 4);
        assert_eq!(c.color, ColorMode::Auto);
        assert!(c.custom_indicator.is_none());
        assert!(!c.ampm && !c.nerd && !c.hide_details && !c.fahrenheit);
    }

    #[test]
    fn serializes_compactly() {
        let json = serde_json::to_string(&Config::default()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["lang"], "EN");
        assert_eq!(v["yticks"], 4);
    }
}
