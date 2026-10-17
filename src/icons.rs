//! Weather icon mappings: emoji, nerd font symbols, and ASCII art.

use serde::{Deserialize, Serialize};

pub const ICON_PLACEHOLDER: &str = "{ICON}";
pub const CLOUD_COVER_ICON: &str = "\u{2601}\u{fe0f}";
pub const PRECIPITATION_ICON: &str = "\u{1F327}\u{fe0f}";
pub const VISIBILITY_ICON: &str = "\u{1F441}\u{fe0f}";
pub const ERROR_ICON: &str = "\u{26D3}\u{fe0f}";

const WEATHER_CODES: &[(i64, &str)] = &[
    (0, "\u{2600}\u{fe0f}"),
    (1, "\u{2600}\u{fe0f}"),
    (2, "\u{26C5}"),
    (3, "\u{2601}\u{fe0f}"),
    (4, "\u{1F325}\u{fe0f}"),
    (5, "\u{1F32B}\u{fe0f}"),
    (10, "\u{1F32B}\u{fe0f}"),
    (45, "\u{1F32B}\u{fe0f}"),
    (60, "\u{1F327}\u{fe0f}"),
    (61, "\u{1F327}\u{fe0f}"),
    (63, "\u{1F327}\u{fe0f}"),
    (80, "\u{1F327}\u{fe0f}"),
    (95, "\u{26C8}\u{fe0f}"),
    (97, "\u{26C8}\u{fe0f}"),
];

const FALLBACK_ICON: &str = "\u{2600}\u{fe0f}";

const WEATHER_CODES_NERD: &[(i64, &str)] = &[
    (0, "\u{F0319}"),
    (1, "\u{F0319}"),
    (2, "\u{F0315}"),
    (3, "\u{F0330}"),
    (4, "\u{F0310}"),
    (5, "\u{F0311}"),
    (10, "\u{F0311}"),
    (45, "\u{F0311}"),
    (60, "\u{F0317}"),
    (61, "\u{F0317}"),
    (63, "\u{f0316}"),
    (80, "\u{f0316}"),
    (95, "\u{F033E}"),
    (97, "\u{F033E}"),
];

const FALLBACK_ICON_NERD: &str = "\u{F0319}";

type AsciiIcon = [&'static str; 5];

const CODE_UNKNOWN: AsciiIcon = [
    "    .-.      ",
    "     __)     ",
    "    (        ",
    "     `-᾿     ",
    "      •      ",
];

const CODE_CLOUDY: AsciiIcon = [
    "             ",
    "{38;5;250}     .--.    ",
    "{38;5;250}  .-(    ).  ",
    "{38;5;250} (___.__)__) ",
    "             ",
];

const CODE_FOG: AsciiIcon = [
    "             ",
    "{38;5;251} _ - _ - _ - ",
    "{38;5;251}  _ - _ - _  ",
    "{38;5;251} _ - _ - _ - ",
    "             ",
];

const CODE_HEAVY_RAIN: AsciiIcon = [
    "{38;5;244;1}     .-.     ",
    "{38;5;244;1}    (   ).   ",
    "{38;5;244;1}   (___(__)  ",
    "{38;5;33;1}  ‚ʻ‚ʻ‚ʻ‚ʻ   ",
    "{38;5;33;1}  ‚ʻ‚ʻ‚ʻ‚ʻ   ",
];

const CODE_LIGHT_RAIN: AsciiIcon = [
    "{38;5;250}     .-.     ",
    "{38;5;250}    (   ).   ",
    "{38;5;250}   (___(__)  ",
    "{38;5;111}    ʻ ʻ ʻ ʻ  ",
    "{38;5;111}   ʻ ʻ ʻ ʻ   ",
];

const CODE_PARTLY_CLOUDY: AsciiIcon = [
    "{38;5;226}   \\__/      ",
    "{38;5;226} __/  {38;5;250}.-.    ",
    "{38;5;226}   \\_{38;5;250}(   ).  ",
    "{38;5;226}   /{38;5;250}(___(__) ",
    "             ",
];

const CODE_SUNNY: AsciiIcon = [
    "{38;5;226}    \\ . /    ",
    "{38;5;226}   - .-. -   ",
    "{38;5;226}  ‒ (   ) ‒  ",
    "{38;5;226}   . `-᾿ .   ",
    "{38;5;226}    / ' \\    ",
];

const CODE_THUNDERY_HEAVY_RAIN: AsciiIcon = [
    "{38;5;244;1}     .-.     ",
    "{38;5;244;1}    (   ).   ",
    "{38;5;244;1}   (___(__)  ",
    "{38;5;33;1}  ‚ʻ{38;5;228;1}⚡{38;5;33;1}ʻ‚{38;5;228;1}⚡{38;5;33;1}‚ʻ   ",
    "{38;5;33;1}  ‚ʻ‚ʻ{38;5;228;1}⚡{38;5;33;1}ʻ‚ʻ   ",
];

const CODE_VERY_CLOUDY: AsciiIcon = [
    "             ",
    "{38;5;244;1}     .--.    ",
    "{38;5;244;1}  .-(    ).  ",
    "{38;5;244;1} (___.__)__) ",
    "             ",
];

/// Icon variant selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconSet {
    Emoji,
    Nerd,
}

impl IconSet {
    pub fn from_nerd(nerd: bool) -> Self {
        if nerd {
            IconSet::Nerd
        } else {
            IconSet::Emoji
        }
    }
}

/// Map a BMKG weather code to an emoji or nerd-font glyph.
pub fn weather_icon(code: i64, nerd: bool) -> &'static str {
    let table = if nerd {
        WEATHER_CODES_NERD
    } else {
        WEATHER_CODES
    };
    table
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, s)| *s)
        .unwrap_or(if nerd {
            FALLBACK_ICON_NERD
        } else {
            FALLBACK_ICON
        })
}

/// Wind-direction arrow for degrees, rounded to 8 compass points.
/// Negative degrees wrap (e.g. -45 behaves like 315/NW).
pub fn wind_dir_icon(degrees: i64, nerd: bool) -> &'static str {
    let dir = ((degrees % 360) as f64 / 45.0).round().rem_euclid(8.0) as usize;
    if nerd {
        [
            "\u{F0340}",
            "\u{F0347}",
            "\u{F0343}",
            "\u{F0349}",
            "\u{F0341}",
            "\u{F0348}",
            "\u{F0342}",
            "\u{F0346}",
        ][dir]
    } else {
        [
            "\u{2B06}\u{fe0f}",
            "\u{2197}\u{fe0f}",
            "\u{27A1}\u{fe0f}",
            "\u{2198}\u{fe0f}",
            "\u{2B07}\u{fe0f}",
            "\u{2199}\u{fe0f}",
            "\u{2B05}\u{fe0f}",
            "\u{2196}\u{fe0f}",
        ][dir]
    }
}

/// 5-line ASCII art for a weather code (lines may carry `{sgr}` color tokens).
pub fn ascii_icon(code: i64) -> AsciiIcon {
    match code {
        0 | 1 => CODE_SUNNY,
        2 => CODE_PARTLY_CLOUDY,
        3 => CODE_CLOUDY,
        4 => CODE_VERY_CLOUDY,
        5 | 10 | 45 => CODE_FOG,
        60 | 61 => CODE_LIGHT_RAIN,
        63 | 80 => CODE_HEAVY_RAIN,
        95 | 97 => CODE_THUNDERY_HEAVY_RAIN,
        _ => CODE_UNKNOWN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_icon_known_codes() {
        assert_eq!(weather_icon(0, false), "\u{2600}\u{fe0f}");
        assert_eq!(weather_icon(2, false), "\u{26C5}");
        assert_eq!(weather_icon(3, false), "\u{2601}\u{fe0f}");
        assert_eq!(weather_icon(95, true), "\u{F033E}");
    }

    #[test]
    fn weather_icon_fallback() {
        assert_eq!(weather_icon(99, false), FALLBACK_ICON);
        assert_eq!(weather_icon(99, true), FALLBACK_ICON_NERD);
    }

    #[test]
    fn wind_dir_all_cardinals() {
        assert_eq!(wind_dir_icon(0, false), "\u{2B06}\u{fe0f}");
        assert_eq!(wind_dir_icon(45, false), "\u{2197}\u{fe0f}");
        assert_eq!(wind_dir_icon(90, false), "\u{27A1}\u{fe0f}");
        assert_eq!(wind_dir_icon(135, false), "\u{2198}\u{fe0f}");
        assert_eq!(wind_dir_icon(180, false), "\u{2B07}\u{fe0f}");
        assert_eq!(wind_dir_icon(225, false), "\u{2199}\u{fe0f}");
        assert_eq!(wind_dir_icon(270, false), "\u{2B05}\u{fe0f}");
        assert_eq!(wind_dir_icon(315, false), "\u{2196}\u{fe0f}");
    }

    #[test]
    fn wind_dir_all_nerd() {
        assert_eq!(wind_dir_icon(0, true), "\u{F0340}");
        assert_eq!(wind_dir_icon(90, true), "\u{F0343}");
        assert_eq!(wind_dir_icon(180, true), "\u{F0341}");
        assert_eq!(wind_dir_icon(270, true), "\u{F0342}");
    }

    #[test]
    fn wind_dir_wraps_indices() {
        assert_eq!(wind_dir_icon(360, false), wind_dir_icon(0, false));
        assert_eq!(wind_dir_icon(405, false), wind_dir_icon(45, false));
        assert_eq!(wind_dir_icon(-45, false), wind_dir_icon(315, false));
    }

    #[test]
    fn ascii_icon_is_five_lines() {
        for code in [0i64, 2, 3, 4, 5, 60, 63, 95, 999] {
            assert_eq!(ascii_icon(code).len(), 5, "code {code}");
        }
    }

    #[test]
    fn icon_set_conversion() {
        assert_eq!(IconSet::from_nerd(true), IconSet::Nerd);
        assert_eq!(IconSet::from_nerd(false), IconSet::Emoji);
    }
}
