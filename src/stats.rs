//! Pure statistics over archived forecasts.
//!
//! I/O consumers load JSONL archive rows into [`ArchiveEntry`] values and call
//! [`summarize`]; rendering to a table or JSON happens with [`StatRow`]
//! formatters so this module stays free of filesystem and stdout concerns.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

use crate::model::Forecast;

/// One archived forecast snapshot.
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub fetched_at: DateTime<Utc>,
    pub adm4: String,
    pub forecast: Forecast,
}

/// Variables supported in stats windows.
pub const KNOWN_VARIABLES: &[(&str, &str)] = &[
    ("t", "Temp(°C)"),
    ("hu", "Hum(%)"),
    ("tp", "Precip(mm)"),
    ("ws", "Wind(km/h)"),
    ("tcc", "Cloud(%)"),
];

/// Options controlling aggregation.
#[derive(Debug, Clone, Default)]
pub struct StatsOpts {
    /// Only include forecasts for a specific adm4 code (None = all).
    pub adm4: Option<String>,
    /// Restrict slots to dates >= start.
    pub start: Option<NaiveDate>,
    /// Restrict slots to dates <= end.
    pub end: Option<NaiveDate>,
    /// Variables to aggregate (in order), e.g. `["t", "hu", "tp", "ws", "tcc"]`.
    pub variables: Vec<String>,
}

/// Statistics for one variable within a (date, hour) bucket.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct VarStats {
    pub mean: f64,
    pub stddev: f64,
    pub n: u64,
}

impl VarStats {
    /// std indexed 0 when there are fewer than 2 samples.
    pub fn stddev_display(&self) -> f64 {
        if self.n < 2 {
            0.0
        } else {
            self.stddev
        }
    }
}

/// One (date, hour) bucket of aggregated variables.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatRow {
    pub date: String,
    pub hour: i32,
    pub lead_avg: f64,
    #[serde(flatten)]
    pub vars: BTreeMap<String, VarStats>,
}

#[derive(Debug, Default)]
struct Agg {
    count: u64,
    sum: f64,
    sum_sq: f64,
    lead_sum: f64,
}

impl Agg {
    fn update(&mut self, value: f64, lead_hours: f64) {
        self.count += 1;
        self.sum += value;
        self.sum_sq += value * value;
        self.lead_sum += lead_hours;
    }

    fn finish(&self) -> VarStats {
        let mean = if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        };
        let stddev = if self.count < 2 {
            0.0
        } else {
            let variance =
                (self.sum_sq - self.sum * self.sum / self.count as f64) / (self.count as f64 - 1.0);
            variance.sqrt()
        };
        VarStats {
            mean,
            stddev,
            n: self.count,
        }
    }

    fn mean_lead(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.lead_sum / self.count as f64
        }
    }
}

fn slot_value(slot: &crate::model::Slot, var: &str) -> Option<f64> {
    match var {
        "t" => Some(slot.t as f64),
        "hu" => Some(slot.hu as f64),
        "tcc" => Some(slot.tcc as f64),
        "tp" => Some(slot.tp),
        "ws" => Some(slot.ws),
        _ => None,
    }
}

/// Aggregate archived forecasts into (date, hour) rows sorted chronologically.
pub fn summarize(entries: &[ArchiveEntry], opts: &StatsOpts) -> Vec<StatRow> {
    let mut data: HashMap<(NaiveDate, u16), BTreeMap<String, Agg>> = HashMap::new();

    for entry in entries {
        if let Some(ref filter) = opts.adm4 {
            if entry.adm4 != *filter {
                continue;
            }
        }

        for slot in entry.forecast.slots() {
            let (Some(slot_date), Some(slot_hour), Some(slot_utc)) =
                (slot.local_date(), slot.local_hour(), slot.utc_dt())
            else {
                continue;
            };
            if let Some(start) = opts.start {
                if slot_date < start {
                    continue;
                }
            }
            if let Some(end) = opts.end {
                if slot_date > end {
                    continue;
                }
            }
            let lead_hours =
                (entry.fetched_at.naive_utc() - slot_utc).num_seconds() as f64 / 3600.0;

            let key = (slot_date, slot_hour as u16);
            let buckets = data.entry(key).or_default();
            for var in &opts.variables {
                if let Some(val) = slot_value(slot, var) {
                    let agg = buckets.entry(var.clone()).or_default();
                    agg.update(val, lead_hours);
                }
            }
        }
    }

    let mut rows: Vec<StatRow> = data
        .iter()
        .map(|((date, hour), buckets)| {
            let mut row = StatRow {
                date: date.format("%Y-%m-%d").to_string(),
                hour: *hour as i32,
                lead_avg: 0.0,
                vars: BTreeMap::new(),
            };
            if let Some(first) = buckets.values().next() {
                row.lead_avg = first.mean_lead();
            }
            for (var, agg) in buckets {
                row.vars.insert(var.clone(), agg.finish());
            }
            row
        })
        .collect();
    rows.sort_by_key(|r| (r.date.clone(), r.hour));
    rows
}

fn vars_header(variables: &[String]) -> String {
    variables
        .iter()
        .map(|v| {
            KNOWN_VARIABLES
                .iter()
                .find(|(k, _)| *k == v.as_str())
                .map(|(_, label)| label.to_string())
                .unwrap_or_else(|| v.clone())
        })
        .collect::<Vec<_>>()
        .join("  ")
}

/// Render rows as the fixed-width stats table (identical layout to the old
/// `cuaca stats` text output). Trailing newline included.
pub fn to_table(rows: &[StatRow], variables: &[String]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{:<12} {:<6} {:<8} {}",
        "Date",
        "Hour",
        "Lead(h)",
        vars_header(variables)
    ));
    out.push('\n');
    out.push_str(&"-".repeat(12 + 1 + 6 + 1 + 8 + 1 + 50));
    out.push('\n');
    for row in rows {
        let var_parts: Vec<String> = variables
            .iter()
            .map(|var| match row.vars.get(var) {
                Some(stats) => format!(
                    "{} {:.1}\u{b1}{:.1}",
                    var,
                    stats.mean,
                    stats.stddev_display()
                ),
                None => format!("{} N/A", var),
            })
            .collect();
        out.push_str(&format!(
            "{:<12} {:<6} {:<8.1} {}",
            row.date,
            row.hour,
            row.lead_avg,
            var_parts.join("  ")
        ));
        out.push('\n');
    }
    out
}

/// Render rows as pretty JSON (same field layout as the old `cuaca stats --format json`).
pub fn to_json(rows: &[StatRow]) -> Result<String, crate::error::CuacaError> {
    serde_json::to_string_pretty(rows).map_err(crate::error::CuacaError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Location, Slot, SlotGroup};

    fn slot(local: &str, utc: &str, values: (i64, i64, i64, f64, f64)) -> Slot {
        Slot {
            local_datetime: Some(local.to_string()),
            utc_datetime: Some(utc.to_string()),
            t: values.0,
            hu: values.1,
            tcc: values.2,
            tp: values.3,
            ws: values.4,
            ..Slot::default()
        }
    }

    fn entry(adm4: &str, fetched_at: &str, slots: Vec<Slot>) -> ArchiveEntry {
        ArchiveEntry {
            fetched_at: DateTime::parse_from_rfc3339(fetched_at)
                .unwrap()
                .with_timezone(&Utc),
            adm4: adm4.to_string(),
            forecast: Forecast {
                location: Location::default(),
                groups: vec![SlotGroup { slots }],
            },
        }
    }

    fn variables() -> Vec<String> {
        ["t", "hu", "tp", "ws", "tcc"]
            .iter()
            .map(|v| v.to_string())
            .collect()
    }

    #[test]
    fn test_agg_mean() {
        let mut agg = Agg::default();
        agg.update(10.0, 1.0);
        agg.update(20.0, 2.0);
        agg.update(30.0, 3.0);
        let stats = agg.finish();
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.n, 3);
        assert_eq!(agg.mean_lead(), 2.0);
    }

    #[test]
    fn test_agg_stddev() {
        let mut agg = Agg::default();
        agg.update(2.0, 0.0);
        agg.update(4.0, 0.0);
        agg.update(6.0, 0.0);
        // variance = ((4-4)+(0)+(4))/2 = 4, stddev=2
        assert!((agg.finish().stddev - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_agg_stddev_single_sample() {
        let mut agg = Agg::default();
        agg.update(5.0, 0.0);
        let s = agg.finish();
        assert_eq!(s.stddev, 0.0);
        assert_eq!(s.stddev_display(), 0.0);
        assert_eq!(s.n, 1);
    }

    #[test]
    fn test_agg_mean_lead() {
        let mut agg = Agg::default();
        agg.update(0.0, 1.0);
        agg.update(0.0, 3.0);
        agg.update(0.0, 5.0);
        assert_eq!(agg.mean_lead(), 3.0);
    }

    #[test]
    fn summarize_buckets_and_leads() {
        let entries = vec![entry(
            "31.71.03.1001",
            "2026-05-10T06:00:00Z",
            vec![
                slot(
                    "2026-05-10 13:00:00",
                    "2026-05-10 06:00:00",
                    (30, 70, 20, 0.0, 5.0),
                ),
                slot(
                    "2026-05-10 13:00:00",
                    "2026-05-10 06:00:00",
                    (32, 78, 30, 4.0, 9.0),
                ),
                slot(
                    "2026-05-11 02:00:00",
                    "2026-05-10 19:00:00",
                    (28, 80, 40, 1.0, 3.0),
                ),
            ],
        )];

        let rows = summarize(
            &entries,
            &StatsOpts {
                variables: variables(),
                ..StatsOpts::default()
            },
        );

        assert_eq!(rows.len(), 2);
        let first = &rows[0];
        assert_eq!(first.date, "2026-05-10");
        assert_eq!(first.hour, 13);
        assert_eq!(first.lead_avg, 0.0); // both fetched exactly at slot time
        assert_eq!(first.vars["t"].mean, 31.0);
        assert!((first.vars["ws"].mean - 7.0).abs() < 1e-9);
        let second = &rows[1];
        assert_eq!(second.date, "2026-05-11");
        assert_eq!(second.hour, 2);
        assert_eq!(second.lead_avg, -13.0); // fetched before slot time (06:00Z vs 19:00Z)
    }

    #[test]
    fn summarize_filters_adm4_and_dates() {
        let a = entry(
            "31.71.03.1001",
            "2026-05-10T06:00:00Z",
            vec![slot(
                "2026-05-10 13:00:00",
                "2026-05-10 06:00:00",
                (30, 70, 20, 0.0, 5.0),
            )],
        );
        let b = entry(
            "32.73.07.1001",
            "2026-05-10T06:00:00Z",
            vec![slot(
                "2026-06-01 13:00:00",
                "2026-06-01 06:00:00",
                (20, 80, 60, 10.0, 2.0),
            )],
        );

        let rows = summarize(
            &[a.clone(), b.clone()],
            &StatsOpts {
                adm4: Some("31.71.03.1001".to_string()),
                variables: variables(),
                ..StatsOpts::default()
            },
        );
        assert_eq!(rows.len(), 1);

        let rows = summarize(
            &[a, b],
            &StatsOpts {
                variables: variables(),
                start: Some(NaiveDate::from_ymd_opt(2026, 5, 11).unwrap()),
                ..StatsOpts::default()
            },
        );
        // a's slots (2026-05-10) fall below `start` and are skipped;
        // b's slot on 2026-06-01 is kept.
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].date, "2026-06-01");
    }

    #[test]
    fn table_render_matches_old_layout() {
        let entries = vec![entry(
            "31.71.03.1001",
            "2026-05-10T06:00:00Z",
            vec![slot(
                "2026-05-10 13:00:00",
                "2026-05-10 06:00:00",
                (30, 70, 20, 0.0, 5.0),
            )],
        )];
        let rows = summarize(
            &entries,
            &StatsOpts {
                variables: variables(),
                ..StatsOpts::default()
            },
        );
        let table = to_table(&rows, &variables());
        let lines: Vec<&str> = table.lines().collect();
        assert!(lines[0].contains("Date"));
        assert!(lines[0].contains("Temp(°C)"));
        assert_eq!(lines[1], "-".repeat(79));
        let row = lines[2];
        assert!(row.starts_with("2026-05-10"));
        assert!(row.contains("t 30.0±0.0")); // single sample → stddev displayed 0
        assert!(row.contains("hu 70.0±0.0"));
        assert!(row.contains("ws 5.0±0.0"));
    }

    #[test]
    fn json_render_has_expected_fields() {
        let entries = vec![entry(
            "31.71.03.1001",
            "2026-05-10T06:00:00Z",
            vec![slot(
                "2026-05-10 13:00:00",
                "2026-05-10 06:00:00",
                (30, 70, 20, 0.0, 5.0),
            )],
        )];
        let rows = summarize(
            &entries,
            &StatsOpts {
                variables: variables(),
                ..StatsOpts::default()
            },
        );
        let json = to_json(&rows).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let row = &v[0];
        assert_eq!(row["date"], "2026-05-10");
        assert_eq!(row["hour"], 13);
        assert_eq!(row["t"]["mean"], 30.0);
        assert_eq!(row["t"]["n"], 1);
    }
}
