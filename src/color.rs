//! ANSI colorization and Pango conversion utilities.
//!
//! A single color-ready flag drives all paint helpers; the ASCII art carries
//! `{sgr}` color tokens that are expanded when color is enabled and stripped
//! otherwise. [`ansi_to_pango`] converts the ANSI stream into Pango markup.

use serde::{Deserialize, Serialize};
use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

static COLOR_ENABLED: AtomicBool = AtomicBool::new(true);

/// Apply a color mode: `Always`/`Never` take effect immediately;
/// `Auto` consults `NO_COLOR` and stdout tty-ness.
pub fn set_color_mode(mode: ColorMode) {
    let enabled = match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            if std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty()) {
                false
            } else {
                std::io::stdout().is_terminal()
            }
        }
    };
    COLOR_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Force color on/off directly (useful for tests and non-stdio consumers).
pub fn set_color_enabled(enabled: bool) {
    COLOR_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn is_color() -> bool {
    COLOR_ENABLED.load(Ordering::Relaxed)
}

const RESET: &str = "\x1b[0m";

fn paint(text: &str, code: &str) -> String {
    if !is_color() {
        return text.to_string();
    }
    format!("{}{}{}", code, text, RESET)
}

pub fn temp_line(text: &str, temp: i64) -> String {
    if !is_color() {
        return text.to_string();
    }
    let code = if temp <= 10 {
        "\x1b[38;5;33m"
    } else if temp <= 20 {
        "\x1b[38;5;51m"
    } else if temp <= 25 {
        "\x1b[38;5;49m"
    } else if temp <= 28 {
        "\x1b[38;5;226m"
    } else if temp <= 32 {
        "\x1b[38;5;214m"
    } else {
        "\x1b[38;5;196m"
    };
    format!("{}{}{}", code, text, RESET)
}

pub fn rain_bar(text: &str) -> String {
    paint(text, "\x1b[38;5;27m")
}

pub fn cloud_bar(text: &str) -> String {
    paint(text, "\x1b[38;5;244m")
}

pub fn humid_spark(text: &str) -> String {
    paint(text, "\x1b[38;5;129m")
}

pub fn wind_spark(text: &str) -> String {
    paint(text, "\x1b[38;5;46m")
}

pub fn vis_spark(text: &str) -> String {
    paint(text, "\x1b[38;5;51m")
}

pub fn weather_icon_line(line: &str, code: i64) -> String {
    if !is_color() {
        return strip_ansi(line);
    }
    // If line already contains ANSI codes, assume it's pre-colored and return as-is.
    if line.contains('\x1b') {
        return line.to_string();
    }
    let color_code = match code {
        0 | 1 => "\x1b[38;5;226m",
        2 => "\x1b[38;5;220m",
        3 | 4 => "\x1b[38;5;248m",
        5 | 10 | 45 => "\x1b[38;5;248m",
        60 | 61 => "\x1b[38;5;51m",
        63 | 80 => "\x1b[38;5;27m",
        95 | 97 => "\x1b[38;5;201m",
        _ => "\x1b[38;5;250m",
    };
    format!("{}{}{}", color_code, line, RESET)
}

pub fn desc_text(text: &str, code: i64) -> String {
    weather_icon_line(text, code)
}

pub fn header(text: &str) -> String {
    paint(text, "\x1b[1;37m")
}

pub fn dim(text: &str) -> String {
    paint(text, "\x1b[2m")
}

pub fn warning(text: &str) -> String {
    if !is_color() {
        return text.to_string();
    }
    paint(text, "\x1b[38;5;226m")
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for c2 in chars.by_ref() {
                if c2 == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Compute hex color for ANSI 256-color index.
fn ansi256_to_hex(n: u8) -> String {
    let basic: [&str; 16] = [
        "#000000", "#800000", "#008000", "#808000", "#000080", "#800080", "#008080", "#c0c0c0",
        "#808080", "#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff",
    ];
    if n < 16 {
        return basic[n as usize].to_string();
    }
    if n < 232 {
        let idx = n as usize - 16;
        let r = idx / 36;
        let g = (idx % 36) / 6;
        let b = idx % 6;
        let vals = [0, 95, 135, 175, 215, 255];
        let (rv, gv, bv) = (vals[r], vals[g], vals[b]);
        return format!("#{:02x}{:02x}{:02x}", rv, gv, bv);
    }
    let shade = (n as usize - 232) * 10 + 8;
    format!("#{:02x}{:02x}{:02x}", shade, shade, shade)
}

/// Convert ANSI escape sequences (256-color) to Pango markup.
/// Supports: reset (0), foreground 256-color (38;5;N). Ignores unsupported codes.
pub fn ansi_to_pango(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let mut current_fg: Option<u8> = None;

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            let mut seq = String::new();
            for c2 in chars.by_ref() {
                seq.push(c2);
                if c2 == 'm' {
                    break;
                }
            }
            if let Some(inner) = seq.strip_prefix('[') {
                let inner = inner.trim_end_matches('m');
                if inner.is_empty() || inner == "0" {
                    if current_fg.is_some() {
                        result.push_str("</span>");
                        current_fg = None;
                    }
                    continue;
                }
                let parts: Vec<Option<u8>> =
                    inner.split(';').map(|s| s.parse::<u8>().ok()).collect();
                let mut i = 0;
                while i < parts.len() {
                    let mut parts_matched = 0;
                    match parts[i] {
                        Some(0) if current_fg.is_some() => {
                            result.push_str("</span>");
                            current_fg = None;
                        }
                        Some(38)
                            if i + 2 < parts.len()
                                && parts[i + 1] == Some(5)
                                && parts[i + 2].is_some() =>
                        {
                            let n = parts[i + 2].unwrap();
                            if current_fg != Some(n) {
                                if current_fg.is_some() {
                                    result.push_str("</span>");
                                }
                                let hex = ansi256_to_hex(n);
                                result.push_str(&format!(r#"<span foreground="{}">"#, hex));
                                current_fg = Some(n);
                            }
                            parts_matched = 2;
                        }
                        Some(48) if i + 2 < parts.len() => {
                            parts_matched = 2;
                        }
                        _ => {}
                    }
                    i += 1 + parts_matched;
                }
            }
        } else {
            result.push(c);
        }
    }
    if current_fg.is_some() {
        result.push_str("</span>");
    }
    result
}

/// Expand `{sgr}` color tokens in a string. Tokens are of the form `{sgr}`
/// where `sgr` is a semicolon-separated list of SGR parameters (e.g.,
/// "38;5;226" or "38;5;244;1"). When color is disabled, tokens are stripped.
pub fn expand_color_tokens(s: &str) -> String {
    if !is_color() {
        let mut result = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '{' {
                for t in chars.by_ref() {
                    if t == '}' {
                        break;
                    }
                }
            } else {
                result.push(c);
            }
        }
        return result;
    }

    let mut out = String::new();
    let mut chars = s.chars().peekable();
    let mut in_color = false;

    while let Some(c) = chars.next() {
        if c == '{' {
            let mut content = String::new();
            for t in chars.by_ref() {
                if t == '}' {
                    break;
                }
                content.push(t);
            }
            if content.chars().all(|ch| ch.is_ascii_digit() || ch == ';') {
                if in_color {
                    out.push_str(RESET);
                }
                out.push('\x1b');
                out.push('[');
                out.push_str(&content);
                out.push('m');
                in_color = true;
            }
        } else {
            out.push(c);
        }
    }

    if in_color {
        out.push_str(RESET);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_ansi_to_pango_256_color() {
        set_color_enabled(true);
        let hex = ansi256_to_hex(226);
        let input = "\x1b[38;5;226mHello\x1b[0m";
        let output = ansi_to_pango(input);
        assert!(output.contains(&format!("foreground=\"{}\"", hex)));
        assert!(output.contains("Hello"));
        assert!(!output.contains('\x1b'));
    }

    #[test]
    #[serial]
    fn test_ansi_to_pango_reset() {
        set_color_enabled(true);
        let input = "\x1b[0mNormal";
        let output = ansi_to_pango(input);
        assert_eq!(output, "Normal");
    }

    #[test]
    #[serial]
    fn test_ansi_to_pango_multiple_colors() {
        set_color_enabled(true);
        let hex1 = ansi256_to_hex(226);
        let hex2 = ansi256_to_hex(51);
        let input = "\x1b[38;5;226mRedish\x1b[38;5;51mBlueish\x1b[0mEnd";
        let output = ansi_to_pango(input);
        assert!(output.contains(&format!("foreground=\"{}\"", hex1)));
        assert!(output.contains(&format!("foreground=\"{}\"", hex2)));
        assert!(output.contains("Redish"));
        assert!(output.contains("Blueish"));
        assert!(output.contains("End"));
    }

    #[test]
    #[serial]
    fn test_ansi_to_pango_ignore_unsupported() {
        set_color_enabled(true);
        let input = "\x1b[38;2;255;0;0mTrueColor\x1b[0m";
        let output = ansi_to_pango(input);
        assert_eq!(output, "TrueColor");
    }

    #[test]
    #[serial]
    fn test_ansi_to_pango_grayscale() {
        set_color_enabled(true);
        assert_eq!(ansi256_to_hex(232), "#080808");
        assert_eq!(ansi256_to_hex(239), "#4e4e4e");
        assert_eq!(ansi256_to_hex(255), "#eeeeee");
    }

    #[test]
    #[serial]
    fn test_expand_color_tokens_single() {
        set_color_enabled(true);
        let input = "Hello {38;5;226}World";
        let output = expand_color_tokens(input);
        assert!(output.starts_with("Hello \x1b[38;5;226m"));
        assert!(output.contains("World"));
        assert!(output.ends_with("\x1b[0m"));
    }

    #[test]
    #[serial]
    fn test_expand_color_tokens_multiple() {
        set_color_enabled(true);
        let input = "{38;5;226}Red{38;5;244}Blue";
        let output = expand_color_tokens(input);
        let expected = "\x1b[38;5;226mRed\x1b[0m\x1b[38;5;244mBlue\x1b[0m";
        assert_eq!(output, expected);
    }

    #[test]
    #[serial]
    fn test_expand_color_tokens_plain() {
        set_color_enabled(true);
        let input = "No tokens here";
        let output = expand_color_tokens(input);
        assert_eq!(output, input);
    }

    #[test]
    #[serial]
    fn test_expand_color_tokens_mixed_invalid() {
        set_color_enabled(true);
        let input = "A{38;5;226}B{XYZ}C";
        let output = expand_color_tokens(input);
        let expected = "A\x1b[38;5;226mBC\x1b[0m";
        assert_eq!(output, expected);
    }

    #[test]
    #[serial]
    fn test_expand_color_tokens_color_disabled() {
        set_color_enabled(false);
        let input = "Hello {38;5;226}World";
        assert_eq!(expand_color_tokens(input), "Hello World");
        set_color_enabled(true);
    }

    #[test]
    #[serial]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[38;5;226mfoo\x1b[0m"), "foo");
    }
}
