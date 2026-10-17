//! Terminal UI renderer using ASCII art and color.
//!
//! Output layout mirrors the old `cuaca --format text` display: a header,
//! an ASCII-art summary block, 3×2 grid of column charts, per-day tables,
//! and the data-source footer.

use chrono::{Locale, NaiveDateTime, Timelike};

use crate::color;
use crate::config::Config;
use crate::format::celsius_to_fahrenheit;
use crate::graphs::column_chart_panel;
use crate::icons::{ascii_icon, wind_dir_icon};
use crate::lang::Lang;
use crate::model::{Forecast, Slot};
use crate::warnings::Warning;

/// Render the multi-line terminal display. Returns the literal
/// "No forecast data available" when the forecast has no slots (parity with
/// the old renderer).
pub fn render(cfg: &Config, forecast: &Forecast, warnings: &[Warning]) -> String {
    color::set_color_mode(cfg.color);
    let lang = cfg.lang;
    let fahrenheit = cfg.fahrenheit;
    let nerd = cfg.nerd;
    let locale = match lang {
        Lang::EN => Locale::en_US,
        Lang::ID => Locale::id_ID,
    };

    let groups_first: &[Slot] = forecast
        .groups
        .first()
        .map(|g| g.slots.as_slice())
        .unwrap_or(&[]);
    let first_slot = match forecast.first_slot() {
        Some(s) => s,
        None => return "No forecast data available".to_string(),
    };
    let lokasi = &forecast.location;
    let location_parts: Vec<&str> = vec![
        lokasi.desa.as_str(),
        lokasi.kecamatan.as_str(),
        lokasi.kotkab.as_str(),
        lokasi.provinsi.as_str(),
    ]
    .into_iter()
    .filter(|p| !p.is_empty())
    .collect();
    let location = location_parts.join(", ");

    // Use the first 8 chronological slots for the column charts (may span days).
    let chart_slots: Vec<&Slot> = forecast
        .groups
        .iter()
        .flat_map(|g| g.slots.iter())
        .take(8)
        .collect();

    let today_slots = groups_first;

    let desc = first_slot.desc(lang);
    let temp_c = first_slot.t;
    let temp = if fahrenheit {
        celsius_to_fahrenheit(temp_c)
    } else {
        temp_c
    };
    let ws = first_slot.ws;
    let wd_deg = first_slot.wd_deg;
    let tp = first_slot.tp;
    let vs_text = first_slot.vs_text.as_deref().unwrap_or("?");

    let first_code = first_slot.weather;
    let icon = ascii_icon(first_code);

    let max_temp_c: f64 = today_slots
        .iter()
        .map(|s| s.t as f64)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(temp_c as f64);
    let max_temp = if fahrenheit {
        celsius_to_fahrenheit(max_temp_c as i64)
    } else {
        max_temp_c as i64
    };

    let first_date = first_slot
        .local_dt()
        .map(|dt| dt.date().format_localized("%a %d %b", locale).to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let mut out = String::new();

    out.push_str(&format!(
        "  {}",
        color::header(&format!("{}: {}", lang.weather_report(), location))
    ));
    out.push('\n');
    out.push_str(&format!("  {}\n", first_date));
    out.push('\n');
    out.push('\n');

    for w in warnings {
        out.push_str(&format!(
            "     \u{26A0}\u{fe0f}  {}\n",
            color::warning(&w.headline)
        ));
    }
    if !warnings.is_empty() {
        out.push('\n');
    }

    let unit = if fahrenheit { "°F" } else { "°C" };

    // Line 1: description
    out.push_str(&format!(
        "     {}   {}\n",
        color::expand_color_tokens(icon[0]),
        color::desc_text(desc, first_code),
    ));
    // Line 2: temperature with max in parentheses
    let temp_str = format!("{:+}({}) {}", temp, max_temp, unit);
    out.push_str(&format!(
        "     {}  {}\n",
        color::expand_color_tokens(icon[1]),
        temp_str,
    ));
    // Line 3: wind speed (no cardinal)
    out.push_str(&format!(
        "     {}  {} {:.0} {}\n",
        color::expand_color_tokens(icon[2]),
        wind_dir_icon(wd_deg, nerd),
        ws as i64,
        lang.wind_unit(),
    ));
    // Line 4: visibility (raw vs_text)
    out.push_str(&format!(
        "     {}  {}\n",
        color::expand_color_tokens(icon[3]),
        vs_text,
    ));
    // Line 5: precipitation
    out.push_str(&format!(
        "     {}  {:.1} mm\n",
        color::expand_color_tokens(icon[4]),
        tp,
    ));
    out.push('\n');

    let conv = |t: f64| -> f64 {
        if fahrenheit {
            celsius_to_fahrenheit(t as i64) as f64
        } else {
            t
        }
    };

    let temps: Vec<f64> = chart_slots.iter().map(|s| conv(s.t as f64)).collect();
    let rains: Vec<f64> = chart_slots.iter().map(|s| s.tp).collect();
    let humids: Vec<f64> = chart_slots.iter().map(|s| s.hu as f64).collect();
    let winds: Vec<f64> = chart_slots.iter().map(|s| s.ws).collect();
    let clouds: Vec<f64> = chart_slots.iter().map(|s| s.tcc as f64).collect();
    let vis_vals: Vec<f64> = chart_slots
        .iter()
        .map(|s| {
            s.vs_text
                .as_deref()
                .and_then(|v| {
                    v.split_whitespace()
                        .find_map(|token| token.parse::<f64>().ok())
                })
                .unwrap_or(0.0)
        })
        .collect();

    let times: Vec<String> = chart_slots.iter().map(|s| hour_of(s)).collect();

    let height = cfg.yticks.clamp(3, 6);

    let mut temp_rows = column_chart_panel(&temps, &times, height, |v| {
        format!("{:.0}°", v.round() as i64)
    });
    let mut rain_rows = column_chart_panel(&rains, &times, height, |v| format!("{:.1}", v));
    let mut humid_rows = column_chart_panel(&humids, &times, height, |v| format!("{:.1}", v));
    let mut wind_rows = column_chart_panel(&winds, &times, height, |v| format!("{:.1}", v));
    let mut cloud_rows = column_chart_panel(&clouds, &times, height, |v| format!("{:.1}", v));
    let mut vis_rows = column_chart_panel(&vis_vals, &times, height, |v| format!("{:.1}", v));

    colorize_temp_panel(&mut temp_rows, &temps);
    colorize_spark_panel(&mut rain_rows, color::rain_bar);
    colorize_spark_panel(&mut humid_rows, color::humid_spark);
    colorize_spark_panel(&mut wind_rows, color::wind_spark);
    colorize_spark_panel(&mut cloud_rows, color::cloud_bar);
    colorize_spark_panel(&mut vis_rows, color::vis_spark);

    let temp_title = if fahrenheit {
        format!("{} (°F)", lang.temperature_base())
    } else {
        format!("{} (°C)", lang.temperature_base())
    };

    render_row(
        &mut out,
        &temp_title,
        &temp_rows,
        lang.rainfall(),
        &rain_rows,
        lang.humidity_label(),
        &humid_rows,
    );
    out.push('\n');
    render_row(
        &mut out,
        lang.wind_label(),
        &wind_rows,
        lang.cloud_label(),
        &cloud_rows,
        lang.visibility_label(),
        &vis_rows,
    );
    out.push('\n');

    // Forecast table for Tomorrow and Day After Tomorrow (skip Today).
    let day_labels = [lang.tomorrow(), lang.day_after_tomorrow()];
    for (g_idx, group) in forecast.groups.iter().enumerate().skip(1).take(2) {
        let date_str = group
            .slots
            .first()
            .and_then(|s| s.local_dt())
            .map(|dt| dt.date().format_localized("%a %d %b", locale).to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let label = day_labels
            .get(g_idx - 1)
            .copied()
            .unwrap_or(lang.day_after_tomorrow());
        out.push_str(&format!(
            "  {}",
            color::header(&format!("{}, {}", label, date_str))
        ));
        out.push('\n');

        out.push_str(&format!(
            "  {}",
            color::dim(&format!(
                "{:>5}  {:<6}  {:<20}  {:<15}  {:>5}  {:>7}  {:<8}",
                "Time", "Temp", "Desc", "Wind", "Cloud", "Precip", "Vis"
            ))
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}",
            color::dim(
                "-----  ------  --------------------  ---------------  -----  -------  --------"
            )
        ));
        out.push('\n');

        for slot in &group.slots {
            let time_str = slot
                .local_dt()
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_else(|| "??:??".to_string());

            let temp_c = slot.t;
            let temp_display = if fahrenheit {
                celsius_to_fahrenheit(temp_c)
            } else {
                temp_c
            };
            let temp_str = format!("{}{}", temp_display, if fahrenheit { "°F" } else { "°C" });

            let desc_str = slot.desc(lang);
            let wind_str = format!("{} {:.0} {}", slot.wd, slot.ws, lang.wind_unit());
            let cloud_str = format!("{:.0}%", slot.tcc as f64);
            let precip_str = format!("{:.1} mm", slot.tp);
            let vis_str = slot.vs_text.as_deref().unwrap_or("?");

            out.push_str(&format!(
                "  {:>5}  {:<6}  {:<20}  {:<15}  {:>5}  {:>7}  {:<8}\n",
                time_str, temp_str, desc_str, wind_str, cloud_str, precip_str, vis_str
            ));
        }
        out.push('\n');
    }

    out.push('\n');
    out.push_str(&format!("  {}", color::dim(lang.source())));
    out.push('\n');

    out
}

/// Chart time label: two-digit hour of `local_datetime`.
fn hour_of(slot: &Slot) -> String {
    let dt_str = slot.local_datetime.as_deref().unwrap_or("");
    if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S") {
        format!("{:02}", dt.hour())
    } else if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%dT%H:%M:%S") {
        format!("{:02}", dt.hour())
    } else if dt_str.len() >= 13 {
        let h = &dt_str[11..13];
        if h.parse::<u32>().is_ok() {
            h.to_string()
        } else {
            "??".to_string()
        }
    } else {
        "??".to_string()
    }
}

pub(crate) fn colorize_temp_panel(rows: &mut [String], temps: &[f64]) {
    if temps.is_empty() {
        return;
    }
    let min_t = temps.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_t = temps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    for i in 0..rows.len() {
        let rev_i = rows.len() - 1 - i;
        let temp_at_row = if rows.len() <= 1 {
            25.0
        } else {
            min_t + (max_t - min_t) * (rev_i as f64 / (rows.len() - 1) as f64)
        };
        rows[i] = color::temp_line(&rows[i], temp_at_row as i64);
    }
}

pub(crate) fn colorize_spark_panel(rows: &mut [String], color_fn: fn(&str) -> String) {
    for row in rows {
        *row = color_fn(row);
    }
}

/// Three-column title + panel row layout.
pub(crate) fn render_row(
    out: &mut String,
    title1: &str,
    panel1: &[String],
    title2: &str,
    panel2: &[String],
    title3: &str,
    panel3: &[String],
) {
    let title_field = |t: &str| -> String {
        let t_truncated: String = if t.chars().count() > 24 {
            t.chars().take(24).collect()
        } else {
            t.to_string()
        };
        let pad_left = 6;
        let graph_w = 24;
        let total_w = pad_left + graph_w;
        let title_len = t_truncated.len();
        let pad = if title_len >= graph_w {
            0
        } else {
            (graph_w - title_len) / 2
        };
        let mut field = " ".repeat(pad_left + pad);
        field.push_str(&t_truncated);
        let remaining = total_w - field.len();
        if remaining > 0 {
            field.push_str(&" ".repeat(remaining));
        }
        field
    };

    out.push_str(&format!(
        "{}  {}  {}\n",
        title_field(title1),
        title_field(title2),
        title_field(title3)
    ));

    let max_h = panel1.len().max(panel2.len()).max(panel3.len());
    for r in 0..max_h {
        let l1 = panel1
            .get(r)
            .map(|s| format!("{:<30}", s))
            .unwrap_or_else(|| " ".repeat(30));
        let l2 = panel2
            .get(r)
            .map(|s| format!("{:<30}", s))
            .unwrap_or_else(|| " ".repeat(30));
        let l3 = panel3
            .get(r)
            .map(|s| format!("{:<30}", s))
            .unwrap_or_else(|| " ".repeat(30));
        out.push_str(&format!("{}  {}  {}\n", l1, l2, l3));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{set_color_enabled, temp_line, ColorMode};
    use crate::config::Config;
    use serial_test::serial;

    fn f() -> Forecast {
        Forecast::parse(
            r#"{
              "lokasi": { "desa": "Kemayoran", "kecamatan": "Kemayoran",
                "kotkab": "Kota Adm. Jakarta Pusat", "provinsi": "DKI Jakarta" },
              "data": [
                { "cuaca": [
                  [
                    { "local_datetime": "2026-05-10 13:00:00", "weather": 1,
                      "weather_desc": "Cerah", "weather_desc_en": "Sunny",
                      "t": 31, "hu": 68, "tcc": 39, "tp": 0.2, "ws": 8.7,
                      "wd_deg": 14, "wd": "N", "vs_text": "< 9 km" },
                    { "local_datetime": "2026-05-10 16:00:00", "weather": 2,
                      "weather_desc": "Berawan Sebagian", "weather_desc_en": "Partly Cloudy",
                      "t": 30, "hu": 70, "tcc": 45, "tp": 0.0, "ws": 6.1,
                      "wd_deg": 90, "wd": "E", "vs_text": "7 km" }
                  ]
                ] }
              ]
            }"#,
        )
        .unwrap()
    }

    fn color_never() -> Config {
        Config {
            color: ColorMode::Never,
            date_format: "%a %d %b".to_string(),
            ..Config::default()
        }
    }

    #[test]
    #[serial]
    fn test_render_terminal_no_data() {
        set_color_enabled(false);
        let empty = Forecast::default();
        let output = render(&color_never(), &empty, &[]);
        assert_eq!(output, "No forecast data available");
    }

    #[test]
    #[serial]
    fn test_render_header_and_summary() {
        set_color_enabled(false);
        let out = render(&color_never(), &f(), &[]);
        assert!(out.contains(
            "Weather Report: Kemayoran, Kemayoran, Kota Adm. Jakarta Pusat, DKI Jakarta"
        ));
        assert!(out.contains("Sun 10 May"));
        assert!(out.contains("Sunny"));
        assert!(!out.contains('\x1b'), "color disabled but escape found");
    }

    #[test]
    #[serial]
    fn test_render_temp_range() {
        set_color_enabled(false);
        let out = render(&color_never(), &f(), &[]);
        assert!(out.contains("+31(31)"), "got: {out}");
        assert!(out.contains("°C"));
    }

    #[test]
    #[serial]
    fn test_render_warns() {
        set_color_enabled(false);
        let w = Warning {
            headline: "Badai Petir".to_string(),
            area_desc: "Banten".to_string(),
            effective: None,
            expires: None,
            polygons: Vec::new(),
            web: None,
        };
        let out = render(&color_never(), &f(), &[w]);
        assert!(out.contains("⚠️"), "missing warning icon");
        assert!(out.contains("Badai Petir"));
    }

    #[test]
    #[serial]
    fn test_render_today_max_in_parens() {
        set_color_enabled(false);
        let mut fc = f();
        fc.groups[0].slots.push(crate::model::Slot {
            local_datetime: Some("2026-05-10 19:00:00".to_string()),
            t: 33,
            weather: 3,
            weather_desc_en: Some("Cloudy".to_string()),
            weather_desc: Some("Berawan".to_string()),
            ..crate::model::Slot::default()
        });
        let out = render(&color_never(), &fc, &[]);
        assert!(out.contains("+31(33)"), "max today temp missing: {out}");
    }

    #[test]
    #[serial]
    fn test_temp_line_colors_by_value() {
        set_color_enabled(true);
        assert!(temp_line("x", 40).starts_with("\x1b[38;5;196m"));
        assert!(temp_line("x", 0).starts_with("\x1b[38;5;33m"));
        assert!(temp_line("x", 12).starts_with("\x1b[38;5;51m"));
        set_color_enabled(false);
        assert_eq!(temp_line("x", 40), "x");
        set_color_enabled(true);
    }

    #[test]
    #[serial]
    fn test_render_row_layout() {
        set_color_enabled(false);
        let mut out = String::new();
        render_row(
            &mut out,
            "Temp",
            &["a".to_string()],
            "Rain",
            &["b".to_string()],
            "Wind",
            &["c".to_string()],
        );
        assert!(out.contains("Temp"));
        assert!(out.contains("Rain"));
        assert!(out.contains("Wind"));
        assert!(out.contains('a'));
        assert!(out.contains('b'));
        assert!(out.contains('c'));
    }

    #[test]
    #[serial]
    fn test_colorize_temp_panel_changes() {
        set_color_enabled(true);
        let temps = vec![20.0, 30.0];
        let mut rows = vec!["25°C".to_string(), "26°C".to_string()];
        colorize_temp_panel(&mut rows, &temps);
        assert_eq!(rows.len(), 2);
        set_color_enabled(false);
    }

    #[test]
    #[serial]
    fn test_hour_variants() {
        set_color_enabled(false);
        let s = Slot {
            local_datetime: Some("2026-05-10T07:30:00".to_string()),
            ..Slot::default()
        };
        assert_eq!(hour_of(&s), "07");
        let s2 = Slot::default();
        assert_eq!(hour_of(&s2), "??");
    }
}
