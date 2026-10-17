//! Formatting helpers: times, temperatures, and `{KEY}` indicator templates.

use chrono::{NaiveDateTime, Timelike};

use crate::icons::{weather_icon, ICON_PLACEHOLDER};
use crate::model::Slot;

pub fn format_time(local_datetime: &str, ampm: bool) -> String {
    let dt =
        NaiveDateTime::parse_from_str(local_datetime, "%Y-%m-%d %H:%M:%S").unwrap_or_else(|_| {
            NaiveDateTime::parse_from_str("2000-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        });
    let hour = dt.time().hour();
    if ampm {
        let am_or_pm = if hour >= 12 { "pm" } else { "am" };
        let hour12 = if hour == 0 || hour == 12 {
            12
        } else {
            hour % 12
        };
        format!("{: <4}", format!("{}{}", hour12, am_or_pm))
    } else {
        format!("{:02}:{:02}", hour, dt.time().minute())
    }
}

pub fn format_temp(temp: i64) -> String {
    format!("{: >3}\u{00b0}", temp)
}

pub fn celsius_to_fahrenheit(celsius: i64) -> i64 {
    (celsius as f64 * 9.0 / 5.0 + 32.0).round() as i64
}

/// Slot field names that can appear as `{KEY}` placeholders.
const SLOT_KEYS: &[&str] = &[
    "local_datetime",
    "utc_datetime",
    "datetime",
    "t",
    "hu",
    "tcc",
    "tp",
    "ws",
    "wd_deg",
    "wd",
    "wd_to",
    "vs",
    "vs_text",
    "weather",
    "weather_desc",
    "weather_desc_en",
    "image",
    "time_index",
    "analysis_date",
];

/// Expand the `{KEY}` placeholders of `expression` using slot fields.
/// `{ICON}` is replaced with the weather icon for the slot's code.
/// Unknown `{KEY}` placeholders are left untouched.
pub fn format_indicator(slot: &Slot, expression: &str, nerd: bool) -> String {
    let mut result = expression.to_string();

    let icon = weather_icon(slot.weather, nerd).to_string();
    result = result.replace(ICON_PLACEHOLDER, &icon);

    for key in SLOT_KEYS {
        let placeholder = format!("{{{}}}", key);
        if let Some(value) = slot.value(key) {
            result = result.replace(&placeholder, &value);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::Lang;

    fn slot() -> Slot {
        crate::model::Slot {
            t: 31,
            hu: 68,
            tcc: 39,
            tp: 0.2,
            ws: 8.7,
            wd_deg: 14,
            wd: "N".to_string(),
            wd_to: Some("S".to_string()),
            vs: Some(8401.0),
            vs_text: Some("< 9 km".to_string()),
            weather: 1,
            weather_desc: Some("Cerah".to_string()),
            weather_desc_en: Some("Sunny".to_string()),
            datetime: Some("2026-05-10T06:00:00Z".to_string()),
            local_datetime: Some("2026-05-10 13:00:00".to_string()),
            image: Some("https://api-apps.bmkg.go.id/storage/icon/cuaca/cerah-am.svg".to_string()),
            analysis_date: Some("2026-05-10T00:00:00".to_string()),
            time_index: Some("5-6".to_string()),
            ..Slot::default()
        }
    }

    #[test]
    fn format_time_24h() {
        assert_eq!(format_time("2026-05-10 13:00:00", false), "13:00");
        assert_eq!(format_time("2026-05-10 00:00:00", false), "00:00");
        assert_eq!(format_time("2026-05-10 06:30:00", false), "06:30");
    }

    #[test]
    fn format_time_ampm() {
        assert_eq!(format_time("2026-05-10 13:00:00", true).trim(), "1pm");
        assert_eq!(format_time("2026-05-10 00:00:00", true).trim(), "12am");
        assert_eq!(format_time("2026-05-10 08:00:00", true).trim(), "8am");
        assert_eq!(format_time("2026-05-10 12:00:00", true).trim(), "12pm");
    }

    #[test]
    fn format_temp_pads_to_three_chars() {
        assert_eq!(format_temp(5), "  5\u{00b0}");
        assert_eq!(format_temp(20), " 20\u{00b0}");
        assert_eq!(format_temp(100), "100\u{00b0}");
        assert_eq!(format_temp(-3), " -3\u{00b0}");
    }

    #[test]
    fn celsius_to_fahrenheit_conversions() {
        assert_eq!(celsius_to_fahrenheit(0), 32);
        assert_eq!(celsius_to_fahrenheit(100), 212);
        assert_eq!(celsius_to_fahrenheit(-40), -40);
        assert_eq!(celsius_to_fahrenheit(37), 99);
        assert_eq!(celsius_to_fahrenheit(20), 68);
    }

    #[test]
    fn custom_indicator_substitutes_placeholder() {
        let result = format_indicator(&slot(), "{t}\u{00b0}C", false);
        assert_eq!(result, "31\u{00b0}C");
    }

    #[test]
    fn custom_indicator_substitutes_icon() {
        let result = format_indicator(&slot(), "{ICON} {t}\u{00b0}C", false);
        assert_eq!(result, "\u{2600}\u{fe0f} 31\u{00b0}C");
    }

    #[test]
    fn custom_indicator_multiple_placeholders() {
        let result = format_indicator(&slot(), "{ICON}{t}\u{00b0}C {weather_desc_en}", false);
        assert_eq!(result, "\u{2600}\u{fe0f}31\u{00b0}C Sunny");
    }

    #[test]
    fn custom_indicator_unknown_key_left_alone() {
        let result = format_indicator(&slot(), "{nosuch}", false);
        assert_eq!(result, "{nosuch}");
    }

    #[test]
    fn custom_indicator_number_placeholder() {
        let s = Slot {
            t: 25,
            hu: 70,
            ..Slot::default()
        };
        let result = format_indicator(&s, "{t} {hu}%", false);
        assert_eq!(result, "25 70%");
    }

    #[test]
    fn custom_indicator_decimal_number() {
        let result = format_indicator(&slot(), "{tp} mm {ws} km/h", false);
        assert_eq!(result, "0.2 mm 8.7 km/h");
    }

    #[test]
    fn desc_fallback_is_question_mark() {
        let s = Slot::default();
        assert_eq!(s.desc(Lang::EN), "?");
        assert_eq!(s.desc(Lang::ID), "?");
    }
}
