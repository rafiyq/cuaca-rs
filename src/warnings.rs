//! Weather warning data model and location matching.
//!
//! This module is pure: CAP XML parsing and fetching live in I/O consumers
//! (see the old `cuaca` crate's `warnings::fetch`) which then feed parsed
//! [`Warning`] values into [`select_relevant`].

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// A BMKG nowcast weather warning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Warning {
    pub headline: String,
    pub area_desc: String,
    pub effective: Option<DateTime<Utc>>,
    pub expires: Option<DateTime<Utc>>,
    pub polygons: Vec<Vec<(f64, f64)>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web: Option<String>,
}

/// Parse a CAP polygon string ("lat,lon lat,lon ...") into vertex pairs.
pub fn parse_polygon(s: &str) -> Vec<(f64, f64)> {
    s.split_whitespace()
        .filter_map(|coord| {
            let parts: Vec<&str> = coord.split(',').collect();
            if parts.len() == 2 {
                let lat = parts[0].parse::<f64>().ok()?;
                let lon = parts[1].parse::<f64>().ok()?;
                Some((lat, lon))
            } else {
                None
            }
        })
        .collect()
}

/// Ray-casting point-in-polygon containment test.
pub fn point_in_polygon(point: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let (x, y) = point;
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        let intersect = ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

impl Warning {
    /// True if any of the warning's polygons contains `point`.
    pub fn contains_point(&self, point: (f64, f64)) -> bool {
        !self.polygons.is_empty()
            && self
                .polygons
                .iter()
                .any(|poly| point_in_polygon(point, poly))
    }

    /// True if the warning is currently valid (or validity is unknown).
    pub fn is_valid_time(&self, now: DateTime<Utc>) -> bool {
        let default_eff = now;
        let default_exp = now + Duration::days(1);
        let eff = self.effective.unwrap_or(default_eff);
        let exp = self.expires.unwrap_or(default_exp);
        now >= eff && now < exp
    }

    /// Province-fallback match: area description contains `province`
    /// (case-insensitive) and the warning is currently valid.
    pub fn matches_province(&self, province: &str, now: DateTime<Utc>) -> bool {
        let province_lower = province.to_ascii_lowercase();
        self.area_desc
            .to_ascii_lowercase()
            .contains(&province_lower)
            && self.is_valid_time(now)
    }
}

/// Select warnings relevant to a location, mirroring the old fetch cascade:
/// polygon containment wins; when no polygon matches, fall back to province
/// name + validity-time filtering.
pub fn select_relevant<'a>(
    alerts: &'a [Warning],
    lat: f64,
    lon: f64,
    province: &str,
    now: DateTime<Utc>,
) -> Vec<&'a Warning> {
    let point = (lat, lon);
    let polygon_hits: Vec<&Warning> = alerts.iter().filter(|w| w.contains_point(point)).collect();
    if !polygon_hits.is_empty() {
        return polygon_hits;
    }
    alerts
        .iter()
        .filter(|w| w.matches_province(province, now))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn warning(headline: &str, area_desc: &str) -> Warning {
        Warning {
            headline: headline.to_string(),
            area_desc: area_desc.to_string(),
            effective: None,
            expires: None,
            polygons: Vec::new(),
            web: None,
        }
    }

    #[test]
    fn test_parse_polygon() {
        let s = " -6.024,106.412 -6.031,106.408 -6.030,106.384 ";
        let pts = parse_polygon(s);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], (-6.024, 106.412));
    }

    #[test]
    fn test_point_in_polygon() {
        let square = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        assert!(point_in_polygon((5.0, 5.0), &square));
        assert!(!point_in_polygon((15.0, 5.0), &square));
    }

    #[test]
    fn test_contains_point() {
        let mut w = warning("h", "Banten");
        assert!(!w.contains_point((5.0, 5.0)));
        w.polygons = vec![vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]];
        assert!(w.contains_point((5.0, 5.0)));
    }

    #[test]
    fn test_validity_time() {
        let now = Utc::now();
        let w = warning("h", "Banten");
        assert!(w.is_valid_time(now));
    }

    #[test]
    fn test_select_relevant_polygon_wins() {
        let mut with_poly = warning("Storm", "Banten");
        with_poly.polygons = vec![vec![(5.0, 5.0), (5.1, 5.1), (5.1, 5.0)]];
        let other = warning("Elsewhere", "Jawa Barat");
        let alerts = [with_poly, other];
        let got = select_relevant(&alerts, 5.05, 5.05, "Nonexistent", Utc::now());
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].headline, "Storm");
    }

    #[test]
    fn test_select_relevant_province_fallback() {
        let w = warning("Hujan Lebat di Banten", "Banten");
        let unrelated = warning("Sunny elsewhere", "Jakarta");
        let alerts = [w, unrelated];
        let got = select_relevant(&alerts, 0.0, 0.0, "banten", Utc::now());
        assert_eq!(got.len(), 1);
        assert!(got[0].headline.contains("Banten"));
    }

    #[test]
    fn test_select_relevant_respects_validity() {
        let past_warn = Warning {
            headline: "Old".to_string(),
            area_desc: "Banten".to_string(),
            effective: None,
            expires: Some(Utc::now() - Duration::hours(2)),
            polygons: Vec::new(),
            web: None,
        };
        let now_warn = Warning {
            headline: "Fresh".to_string(),
            area_desc: "Banten".to_string(),
            effective: None,
            expires: None,
            polygons: Vec::new(),
            web: None,
        };
        let alerts = [past_warn, now_warn];
        let got = select_relevant(&alerts, 0.0, 0.0, "Banten", Utc::now());
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].headline, "Fresh");
    }
}
