//! Column chart panels used by the terminal renderer.

use std::cmp;

const TIME_SLOTS: usize = 8;
const PANEL_GRAPH_W: usize = 24;

/// Generate exactly 8 fixed time labels starting from the first hour (no rounding),
/// incrementing by 3 hours, wrapping at 24. Labels are plain numbers (e.g., "9", "12").
fn generate_time_labels(first_hour: u32) -> Vec<String> {
    let mut labels = Vec::with_capacity(TIME_SLOTS);
    let mut h = first_hour;
    for _ in 0..TIME_SLOTS {
        labels.push(h.to_string());
        h = (h + 3) % 24;
    }
    labels
}

/// Computes the slot index (0..TIME_SLOTS) for a data point's hour relative to base.
fn slot_index(base_hour: u32, hour: u32) -> usize {
    let diff = (hour as i32 - base_hour as i32).rem_euclid(24);
    (diff / 3) as usize
}

/// Draws the time axis with centered labels and a vertical spine at midnight wrap boundaries.
fn time_axis(labels: &[String], width: usize, slot_width: usize) -> String {
    if labels.is_empty() || width == 0 {
        return " ".repeat(width);
    }

    let mut axis = vec![' '; width];

    for (i, label) in labels.iter().enumerate() {
        let slot_start = i * slot_width;
        let center = slot_start + slot_width / 2;
        let start_pos = center.saturating_sub(label.len() / 2);
        for (j, ch) in label.chars().enumerate() {
            if start_pos + j < width {
                axis[start_pos + j] = ch;
            }
        }
    }

    let base = labels[0].parse::<u32>().unwrap_or(0);
    for i in 1..TIME_SLOTS {
        let prev_hour = (base + ((i - 1) as u32) * 3) % 24;
        let curr_hour = (base + (i as u32) * 3) % 24;
        if prev_hour >= 21 && curr_hour < prev_hour {
            let pos = i * slot_width;
            if pos > 0 && pos < width {
                axis[pos] = '│';
            }
        }
    }

    axis.iter().collect()
}

/// Draws a vertical column chart where each Y-axis tick is a separate row.
///
/// - `values`: data points to plot (typically 6-8 points)
/// - `times`: parallel hour strings for X-axis (e.g., "09", "18")
/// - `height`: number of rows (ticks) to display, between 3 and 6
/// - `fmt`: format string for tick labels, e.g., `"{:.0}°C"`
///
/// Returns `height + 1` rows: 0=top, ..., height-1=bottom label, height=time axis.
pub fn column_chart_panel<F>(values: &[f64], times: &[String], height: usize, fmt: F) -> Vec<String>
where
    F: Fn(f64) -> String,
{
    if values.is_empty() || times.is_empty() || height < 2 {
        return vec![];
    }

    let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max_val - min_val;

    let mut grid = vec![vec![' '; PANEL_GRAPH_W]; height];

    let base_hour = times
        .first()
        .and_then(|t| t.trim().parse::<u32>().ok())
        .unwrap_or(0);

    let slot_width = PANEL_GRAPH_W / TIME_SLOTS;

    for (i, &v) in values.iter().enumerate() {
        if i >= times.len() {
            break;
        }
        let hour = times[i].parse::<u32>().unwrap_or(base_hour);
        let slot = slot_index(base_hour, hour);

        let row = if range == 0.0 {
            height - 1
        } else {
            let normalized = (v - min_val) / range;
            (height - 1) - (normalized * (height - 1) as f64).round() as usize
        };

        let col_start = slot * slot_width;
        let col_end = cmp::min(col_start + slot_width, PANEL_GRAPH_W);
        for grid_row in grid.iter_mut().skip(row).take(height - row) {
            for ch in grid_row
                .iter_mut()
                .skip(col_start)
                .take(col_end - col_start)
            {
                *ch = '█';
            }
        }
    }

    let mut rows = Vec::new();
    for row_idx in 0..height {
        let factor = (height - 1 - row_idx) as f64 / (height - 1) as f64;
        let tick_val = if height == 1 {
            min_val
        } else {
            min_val + factor * range
        };
        let label = format!("{:>5}", fmt(tick_val));

        let mut row_str = label;
        row_str.push(' ');
        if row_idx < grid.len() {
            row_str.push_str(&grid[row_idx].iter().collect::<String>());
        } else {
            row_str.push_str(&" ".repeat(PANEL_GRAPH_W));
        }
        rows.push(row_str);
    }

    let time_labels = generate_time_labels(base_hour);
    let ta = time_axis(&time_labels, PANEL_GRAPH_W, slot_width);
    let mut time_row = "     ".to_string();
    time_row.push(' ');
    time_row.push_str(&ta);
    rows.push(time_row);

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_chart_panel_empty() {
        assert!(
            column_chart_panel(&[], &[], 4, |v| format!("{:.0}°", v.round() as i64)).is_empty()
        );
    }

    #[test]
    fn column_chart_panel_single_value() {
        let rows = column_chart_panel(&[25.0], &["20".to_string()], 4, |v| {
            format!("{:.0}°", v.round() as i64)
        });
        assert!(!rows.is_empty());
        assert_eq!(rows.len(), 4 + 1);
    }

    #[test]
    fn column_chart_panel_has_rows() {
        let rows = column_chart_panel(
            &[25.0, 26.0, 27.0, 28.0, 29.0],
            &[
                "20".into(),
                "23".into(),
                "02".into(),
                "05".into(),
                "08".into(),
            ],
            4,
            |v| format!("{:.0}°", v.round() as i64),
        );
        assert!(!rows.is_empty());
        assert_eq!(rows.len(), 4 + 1);
    }

    #[test]
    fn column_chart_panel_labels_correct() {
        let rows = column_chart_panel(
            &[10.0, 20.0, 30.0],
            &["20".into(), "23".into(), "02".into()],
            4,
            |v| format!("{:.1}", v),
        );
        let first_label = rows[0].trim();
        let last_label = rows[3].trim();
        assert!(first_label.starts_with("30"));
        assert!(last_label.starts_with("10"));
    }

    #[test]
    fn time_axis_generates_eight_labels() {
        let labels = generate_time_labels(9);
        assert_eq!(labels.len(), 8);
        assert_eq!(labels[0], "9");
        assert_eq!(labels[1], "12");
        assert_eq!(labels[2], "15");
        assert_eq!(labels[3], "18");
        assert_eq!(labels[4], "21");
        assert_eq!(labels[5], "0");
        assert_eq!(labels[6], "3");
        assert_eq!(labels[7], "6");
    }

    #[test]
    fn time_axis_generates_eight_labels_from_18() {
        let labels = generate_time_labels(18);
        assert_eq!(labels[0], "18");
        assert_eq!(labels[1], "21");
        assert_eq!(labels[2], "0");
        assert_eq!(labels[3], "3");
        assert_eq!(labels[4], "6");
        assert_eq!(labels[5], "9");
        assert_eq!(labels[6], "12");
        assert_eq!(labels[7], "15");
    }

    #[test]
    fn time_axis_places_vertical_spine_at_midnight() {
        let labels = generate_time_labels(18);
        let axis = time_axis(&labels, 24, 3);
        let spine_pos = 6;
        assert_eq!(axis.chars().nth(spine_pos), Some('│'));
    }

    #[test]
    fn time_axis_centers_labels() {
        let labels = generate_time_labels(0);
        let axis = time_axis(&labels, 24, 3);
        assert!(axis.contains('0'));
        let first_slot: String = axis.chars().take(3).collect();
        assert!(first_slot.contains('0'));
    }

    #[test]
    fn slot_index_computes_correctly() {
        assert_eq!(slot_index(9, 9), 0);
        assert_eq!(slot_index(9, 12), 1);
        assert_eq!(slot_index(9, 15), 2);
        assert_eq!(slot_index(9, 18), 3);
        assert_eq!(slot_index(9, 21), 4);
        assert_eq!(slot_index(9, 0), 5);
        assert_eq!(slot_index(9, 3), 6);
        assert_eq!(slot_index(9, 6), 7);
        assert_eq!(slot_index(0, 0), 0);
        assert_eq!(slot_index(0, 21), 7);
        assert_eq!(slot_index(18, 18), 0);
        assert_eq!(slot_index(18, 21), 1);
        assert_eq!(slot_index(18, 0), 2);
    }

    #[test]
    fn column_chart_panel_bars_are_wider() {
        let rows = column_chart_panel(
            &[10.0, 20.0, 30.0, 40.0, 50.0],
            &[
                "9".into(),
                "12".into(),
                "15".into(),
                "18".into(),
                "21".into(),
            ],
            4,
            |v| format!("{:.0}", v),
        );
        assert_eq!(rows.len(), 5);
        let last_row_idx = rows.len() - 2;
        let row = &rows[last_row_idx];
        let blocks: Vec<usize> = row[6..]
            .chars()
            .collect::<Vec<_>>()
            .split(|&c| c != '█')
            .map(|chunk| chunk.len())
            .collect();
        assert!(
            blocks.iter().any(|&len| len >= 3),
            "expected bars of width >=3, got {:?}",
            blocks
        );
    }
}
