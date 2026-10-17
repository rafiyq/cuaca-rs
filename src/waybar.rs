//! Waybar rendering: `{ text, tooltip, class }` for the custom module.

use std::collections::HashMap;

use chrono::{DateTime, Locale, Utc};

use crate::color;
use crate::config::Config;
use crate::error::CuacaError;
use crate::format::{celsius_to_fahrenheit, format_indicator, format_temp, format_time};
use crate::icons::{
    ascii_icon, weather_icon, wind_dir_icon, CLOUD_COVER_ICON, PRECIPITATION_ICON, VISIBILITY_ICON,
};
use crate::lang::Lang;
use crate::model::{Forecast, Slot};
use crate::util::escape_pango;
use crate::warnings::Warning;

/// Waybar custom-module output payload.
#[derive(Debug, Clone, PartialEq)]
pub struct WaybarOut {
    pub text: String,
    pub tooltip: String,
    pub class: String,
}

impl WaybarOut {
    /// Serialize as Waybar JSON (`{"text":…,"tooltip":…,"class":…}`).
    pub fn to_json(&self) -> Result<String, CuacaError> {
        let json = serde_json::json!({
            "text": self.text,
            "tooltip": self.tooltip,
            "class": self.class,
        });
        serde_json::to_string(&json).map_err(CuacaError::from)
    }
}

const NERD_CLOUD: &str = "\u{f0330}";
const NERD_RAIN: &str = "\u{f0317}";
const NERD_EYE: &str = "\u{f02fd}";

fn detail_icons(nerd: bool) -> (&'static str, &'static str, &'static str) {
    if nerd {
        (NERD_CLOUD, NERD_RAIN, NERD_EYE)
    } else {
        (CLOUD_COVER_ICON, PRECIPITATION_ICON, VISIBILITY_ICON)
    }
}

fn format_slot_date(
    slot: &Slot,
    label: &str,
    date_format: &str,
    locale: Locale,
    first_in_body: bool,
) -> String {
    let mut s = if first_in_body {
        "<b>".to_string()
    } else {
        "\n<b>".to_string()
    };
    s += label;
    s += ", ";
    s += &slot
        .local_date()
        .map(|d| d.format_localized(date_format, locale).to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    s += "</b>\n";
    s
}

/// Build the Waybar `{ text, tooltip, class }` output from a forecast.
///
/// Requires non-empty slot data; returns [`CuacaError::Data`] otherwise.
pub fn render(
    cfg: &Config,
    forecast: &Forecast,
    warnings: &[Warning],
) -> Result<WaybarOut, CuacaError> {
    let lang = cfg.lang;
    let nerd = cfg.nerd;

    // Flatten slots, sorted by local_datetime so day headers are chronologically correct.
    let mut flat: Vec<(usize, &Slot)> = forecast
        .groups
        .iter()
        .enumerate()
        .flat_map(|(gi, g)| g.slots.iter().map(move |s| (gi, s)))
        .collect();

    if flat.is_empty() {
        return Err(CuacaError::Data("no forecast data available".to_string()));
    }
    flat.sort_by_key(|(_, s)| s.local_datetime.clone().unwrap_or_default());

    let first_slot = flat[0].1;
    let lokasi = &forecast.location;

    let weather_code = first_slot.weather;
    let bar_icon = weather_icon(weather_code, nerd);
    let weather_desc = first_slot.desc(lang);

    let display_temp = if cfg.fahrenheit {
        celsius_to_fahrenheit(first_slot.t)
    } else {
        first_slot.t
    };
    let unit = if cfg.fahrenheit { "°F" } else { "°C" };

    let mut data = HashMap::new();

    let mut bar_text = match &cfg.custom_indicator {
        None => {
            let indicator = format_temp(display_temp);
            format!("{} {}", bar_icon, indicator)
        }
        Some(expression) => format_indicator(first_slot, expression, nerd),
    };
    if !warnings.is_empty() {
        bar_text = format!("⚠️ {}", bar_text);
    }
    data.insert("text", bar_text);

    let first_temp = first_slot.t;
    let display_temp = if cfg.fahrenheit {
        celsius_to_fahrenheit(first_temp)
    } else {
        first_temp
    };

    let escaped_first_desc = escape_pango(weather_desc);

    let provinsi = lokasi.provinsi.as_str();
    let kotkab = lokasi.kotkab.as_str();
    let kecamatan = lokasi.kecamatan.as_str();
    let desa = lokasi.desa.as_str();
    let location_parts: Vec<&str> = vec![desa, kecamatan, kotkab, provinsi]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();

    let mut tooltip = String::new();
    tooltip.push_str(&format!(
        "<b>{}:</b> {}\n",
        lang.weather_report(),
        escape_pango(&location_parts.join(", "))
    ));
    tooltip.push('\n');
    tooltip.push_str(&format!(
        "<b>{}</b> {} {}\n",
        escaped_first_desc, display_temp, unit
    ));

    // ASCII art column (5 lines) zipped against detail lines for the current slot.
    let raw_ascii = ascii_icon(weather_code);
    let ascii_lines: Vec<String> = raw_ascii
        .iter()
        .map(|&line| color::expand_color_tokens(line))
        .map(|ansi_line| color::ansi_to_pango(&ansi_line))
        .collect();

    let hu_detail = format!("{}: {}%", lang.humidity(), first_slot.hu);
    let tcc_detail = format!("{}: {}%", lang.cloud_cover(), first_slot.tcc);
    let tp_detail = format!("{}: {:.1} mm", lang.precipitation(), first_slot.tp);
    let wind_detail = format!(
        "{}: {} {} {} {}",
        lang.wind(),
        wind_dir_icon(first_slot.wd_deg, nerd),
        first_slot.wd,
        first_slot.ws,
        lang.wind_unit()
    );
    let vs_detail = format!(
        "{}: {}",
        lang.visibility(),
        escape_pango(first_slot.vs_text.as_deref().unwrap_or("?"))
    );

    let detail_lines = [
        &hu_detail,
        &tcc_detail,
        &tp_detail,
        &wind_detail,
        &vs_detail,
    ];

    for (art, detail) in ascii_lines.iter().zip(detail_lines.iter()) {
        let art_mono = if art.is_empty() {
            String::new()
        } else {
            format!(r#"<span font_family="monospace">{}</span>"#, art)
        };
        tooltip.push_str(&format!("{} {}\n", art_mono, detail));
    }

    tooltip.push('\n');

    let locale = match lang {
        Lang::EN => Locale::en_US,
        Lang::ID => Locale::id_ID,
    };

    // Day group headers with localized dates.
    let day_labels = [lang.today(), lang.tomorrow(), lang.day_after_tomorrow()];
    let mut current_group: Option<usize> = None;
    let mut first_header_done = false;
    for (gi, slot) in &flat {
        if current_group.is_none() || current_group != Some(*gi) {
            current_group = Some(*gi);
            let label = day_labels
                .get(*gi)
                .copied()
                .unwrap_or(lang.day_after_tomorrow());
            tooltip.push_str(&format_slot_date(
                slot,
                label,
                cfg.date_format.as_str(),
                locale,
                !first_header_done,
            ));
            first_header_done = true;
        }

        let local_dt = slot.local_datetime.as_deref().unwrap_or("??:??");

        let desc = escape_pango(slot.desc(lang));
        let ws_val = slot.ws;
        let wd_val = slot.wd_deg;
        let temp = if cfg.fahrenheit {
            celsius_to_fahrenheit(slot.t)
        } else {
            slot.t
        };

        let mut line = format!(
            "{}  {}  {}  {}  {} {} {}",
            format_time(local_dt, cfg.ampm),
            weather_icon(slot.weather, nerd),
            format_temp(temp),
            desc,
            wind_dir_icon(wd_val, nerd),
            ws_val,
            lang.wind_unit(),
        );

        if !cfg.hide_details {
            let (cloud_icon, rain_icon, eye_icon) = detail_icons(nerd);
            line += &format!(
                "  {} {}%  {} {} mm  {} {}",
                cloud_icon,
                slot.tcc,
                rain_icon,
                slot.tp,
                eye_icon,
                escape_pango(slot.vs_text.as_deref().unwrap_or("?"))
            );
        }

        line += "\n";
        tooltip += &line;
    }

    if !warnings.is_empty() {
        tooltip += "<b>Weather Warnings:</b>\n";
        for w in warnings {
            let times = match (&w.effective, &w.expires) {
                (Some(e), Some(x)) => {
                    let fmt = |dt: &DateTime<Utc>| dt.format("%H:%M").to_string();
                    format!("Valid: {}–{}\n", fmt(e), fmt(x))
                }
                _ => String::new(),
            };
            tooltip += &format!("• <b>{}<br/>{}", escape_pango(&w.headline), times);
            if let Some(web) = &w.web {
                tooltip += &format!("<a href=\"{}\">Infographic</a>", escape_pango(web));
            }
            tooltip += "</br>\n";
        }
        tooltip += "\n";
    }

    tooltip += &format!("\n<small>{}</small>", lang.source());

    data.insert("tooltip", tooltip);

    let css_class = weather_desc
        .to_lowercase()
        .split(',')
        .next()
        .unwrap_or("unknown")
        .trim()
        .replace(' ', "_");
    data.insert("class", css_class);

    Ok(WaybarOut {
        text: data["text"].clone(),
        tooltip: data["tooltip"].clone(),
        class: data["class"].clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn fixture() -> Forecast {
        Forecast::parse(
            r#"{
              "lokasi": {
                "provinsi": "DKI Jakarta",
                "kotkab": "Kota Adm. Jakarta Pusat",
                "kecamatan": "Kemayoran",
                "desa": "Kemayoran",
                "lat": -6.17172,
                "lon": 106.849983
              },
              "data": [
                { "cuaca": [
                  [ { "local_datetime": "2026-05-10 13:00:00", "weather": 1,
                      "weather_desc": "Cerah", "weather_desc_en": "Sunny",
                      "t": 31, "hu": 68, "tcc": 39, "tp": 0.2, "ws": 8.7,
                      "wd_deg": 14, "wd": "N", "vs_text": "< 9 km" } ],
                  [ { "local_datetime": "2026-05-10 16:00:00", "weather": 2,
                      "weather_desc": "Berawan Sebagian", "weather_desc_en": "Partly Cloudy",
                      "t": 30, "hu": 70, "tcc": 45, "tp": 0.0, "ws": 6.1,
                      "wd_deg": 90, "wd": "E", "vs_text": "7 km" } ]
                ] },
                { "cuaca": [
                  [ { "local_datetime": "2026-05-11 13:00:00", "weather": 60,
                      "weather_desc": "Hujan Ringan", "weather_desc_en": "Light Rain",
                      "t": 27, "hu": 85, "tcc": 90, "tp": 3.5, "ws": 9.4,
                      "wd_deg": 180, "wd": "S", "vs_text": "< 4 km" } ]
                ] }
              ]
            }"#,
        )
        .unwrap()
    }

    fn cfg() -> Config {
        Config {
            ..Config::default()
        }
    }

    #[test]
    #[serial]
    fn render_bar_text() {
        color::set_color_enabled(false);
        let out = render(&cfg(), &fixture(), &[]).unwrap();
        assert_eq!(out.text, "\u{2600}\u{fe0f}  31\u{00b0}");
    }

    #[test]
    #[serial]
    fn render_text_and_class() {
        color::set_color_enabled(false);
        let out = render(&cfg(), &fixture(), &[]).unwrap();
        assert_eq!(out.class, "sunny");
        assert!(out.tooltip.starts_with(
            "<b>Weather Report:</b> Kemayoran, Kemayoran, Kota Adm. Jakarta Pusat, DKI Jakarta\n"
        ));
    }

    #[test]
    #[serial]
    fn render_custom_indicator() {
        color::set_color_enabled(false);
        let mut c = cfg();
        c.custom_indicator = Some("{ICON} {t}°C {weather_desc_en}".to_string());
        let out = render(&c, &fixture(), &[]).unwrap();
        assert_eq!(out.text, "\u{2600}\u{fe0f} 31°C Sunny");
        // no warning prefix
    }

    #[test]
    #[serial]
    fn render_warning_prefix_and_block() {
        color::set_color_enabled(false);
        let w = Warning {
            headline: "Hujan Lebat disertai Petir".to_string(),
            area_desc: "Banten".to_string(),
            effective: Some(
                DateTime::parse_from_rfc3339("2026-05-20T08:05:00+07:00")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            expires: Some(
                DateTime::parse_from_rfc3339("2026-05-20T10:00:00+07:00")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            polygons: Vec::new(),
            web: Some("http://example.com/x.jpg".to_string()),
        };
        let out = render(&cfg(), &fixture(), &[w]).unwrap();
        assert!(out.text.starts_with("⚠️ "));
        assert!(out.tooltip.contains("Weather Warnings:"));
        assert!(out.tooltip.contains("Valid: 01:05–03:00"));
        assert!(out
            .tooltip
            .contains(r#"<a href="http://example.com/x.jpg">Infographic</a>"#));
    }

    #[test]
    #[serial]
    fn render_escapes_pango_in_vs_text() {
        color::set_color_enabled(false);
        let out = render(&cfg(), &fixture(), &[]).unwrap();
        assert!(out.tooltip.contains("&lt; 9 km"));
        assert!(out.tooltip.contains("&lt; 4 km"));
    }

    #[test]
    #[serial]
    fn render_day_headers() {
        color::set_color_enabled(false);
        let c = Config {
            date_format: "%a %d %b".to_string(),
            ..Config::default()
        };
        let out = render(&c, &fixture(), &[]).unwrap();
        assert!(out.tooltip.contains("Today"));
        assert!(out.tooltip.contains("Tomorrow"));
        // 2026-05-10 is a Sunday
        assert!(out.tooltip.contains("Sun 10 May"), "{}", out.tooltip);
    }

    #[test]
    #[serial]
    fn render_fahrenheit() {
        color::set_color_enabled(false);
        let c = Config {
            fahrenheit: true,
            ..Config::default()
        };
        let out = render(&c, &fixture(), &[]).unwrap();
        assert!(out.text.contains("88°")); // 31°C → 88°F
        assert!(out.tooltip.contains("°F"));
    }

    #[test]
    #[serial]
    fn render_ampm_and_hide_details() {
        color::set_color_enabled(false);
        let c = Config {
            ampm: true,
            hide_details: true,
            ..Config::default()
        };
        let out = render(&c, &fixture(), &[]).unwrap();
        assert!(out.tooltip.contains("1pm"));
        assert!(!out.tooltip.contains("%  ")); // details hidden
    }

    #[test]
    #[serial]
    fn render_empty_forecast_errors() {
        color::set_color_enabled(false);
        let empty = Forecast::default();
        let err = render(&cfg(), &empty, &[]).unwrap_err();
        assert!(err.to_string().contains("no forecast data"));
    }

    #[test]
    #[serial]
    fn to_json_serializes_expected_keys() {
        color::set_color_enabled(false);
        let out = render(&cfg(), &fixture(), &[]).unwrap();
        let json = out.to_json().unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["text"].is_string());
        assert!(v["tooltip"].is_string());
        assert_eq!(v["class"], "sunny");
    }
}
