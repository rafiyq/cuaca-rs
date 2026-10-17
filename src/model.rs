//! Typed models for BMKG forecast data.
//!
//! [`Forecast`] deserializes directly from the raw BMKG API payload
//! (`{ "lokasi": ..., "data": [ { "cuaca": [ [slot, ...], ... ] } ] }`)
//! as well as from the canonical form (`{ "location": ..., "groups": [ ... ] }`).

use chrono::{NaiveDate, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};

use crate::error::CuacaError;
use crate::lang::Lang;

/// Location metadata ("lokasi" in BMKG responses).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Location {
    #[serde(default)]
    pub adm1: String,
    #[serde(default)]
    pub adm2: String,
    #[serde(default)]
    pub adm3: String,
    #[serde(default)]
    pub adm4: String,
    #[serde(default)]
    pub provinsi: String,
    #[serde(default)]
    pub kotkab: String,
    #[serde(default)]
    pub kecamatan: String,
    #[serde(default)]
    pub desa: String,
    #[serde(default, deserialize_with = "deserialize_lenient_f64")]
    pub lat: f64,
    #[serde(default, deserialize_with = "deserialize_lenient_f64")]
    pub lon: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

/// A grouping of forecast slots (normally one calendar day).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SlotGroup {
    pub slots: Vec<Slot>,
}

/// One 3-hourly forecast slot.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_datetime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utc_datetime: Option<String>,
    #[serde(default)]
    pub t: i64,
    #[serde(default)]
    pub hu: i64,
    #[serde(default)]
    pub tcc: i64,
    #[serde(default)]
    pub tp: f64,
    #[serde(default)]
    pub ws: f64,
    #[serde(default)]
    pub wd_deg: i64,
    #[serde(default)]
    pub wd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wd_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vs: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vs_text: Option<String>,
    #[serde(default)]
    pub weather: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather_desc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather_desc_en: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_index: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis_date: Option<String>,
}

/// A 3-day BMKG forecast: location metadata plus day groups of hourly slots.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Forecast {
    pub location: Location,
    pub groups: Vec<SlotGroup>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum InputRaw {
    Bmkg {
        #[serde(rename = "lokasi")]
        location: Location,
        #[serde(rename = "data", default)]
        data: Vec<BmkgDay>,
    },
    Canonical {
        #[serde(default)]
        location: Location,
        #[serde(default)]
        groups: Vec<SlotGroup>,
    },
}

#[derive(Deserialize)]
struct BmkgDay {
    #[serde(rename = "cuaca", default)]
    cuaca: Vec<Vec<Slot>>,
}

impl<'de> Deserialize<'de> for Forecast {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = InputRaw::deserialize(deserializer)?;
        Ok(match raw {
            InputRaw::Bmkg { location, data } => Forecast {
                location,
                groups: data
                    .into_iter()
                    .flat_map(|d| d.cuaca)
                    .map(|slots| SlotGroup { slots })
                    .collect(),
            },
            InputRaw::Canonical { location, groups } => Forecast { location, groups },
        })
    }
}

impl Forecast {
    /// Parse a forecast from raw JSON text (BMKG or canonical shape).
    pub fn parse(text: &str) -> Result<Self, CuacaError> {
        serde_json::from_str(text).map_err(CuacaError::from)
    }

    /// Build a forecast from an already-parsed JSON value.
    pub fn from_value(value: serde_json::Value) -> Result<Self, CuacaError> {
        serde_json::from_value(value).map_err(CuacaError::from)
    }

    /// All slots flattened across groups, preserving group order.
    pub fn slots(&self) -> impl Iterator<Item = &Slot> {
        self.groups.iter().flat_map(|g| g.slots.iter())
    }

    /// The chronologically-first slot (bar display / summary source).
    pub fn first_slot(&self) -> Option<&Slot> {
        self.slots().next()
    }

    pub fn is_empty(&self) -> bool {
        self.first_slot().is_none()
    }
}

impl Slot {
    /// Weather description for the given language, with "?" fallback.
    pub fn desc(&self, lang: Lang) -> &str {
        let key = lang.weather_desc_key();
        let value = if key == "weather_desc_en" {
            self.weather_desc_en.as_deref()
        } else {
            self.weather_desc.as_deref()
        };
        value.unwrap_or("?")
    }

    /// Date part of `local_datetime` ("YYYY-MM-DD HH:MM:SS").
    pub fn local_date(&self) -> Option<NaiveDate> {
        self.local_dt().map(|dt| dt.date())
    }

    /// Hour of `local_datetime` in [0, 23].
    pub fn local_hour(&self) -> Option<u32> {
        self.local_dt().map(|dt| dt.time().hour())
    }

    /// Parse `local_datetime` as `%Y-%m-%d %H:%M:%S`.
    pub fn local_dt(&self) -> Option<NaiveDateTime> {
        self.local_datetime
            .as_deref()
            .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
    }

    /// Parse `utc_datetime` as `%Y-%m-%d %H:%M:%S`.
    pub fn utc_dt(&self) -> Option<NaiveDateTime> {
        self.utc_datetime
            .as_deref()
            .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
    }

    /// Lookup a slot field as a display string, for `{KEY}` indicator templates.
    pub fn value(&self, key: &str) -> Option<String> {
        let s = match key {
            "datetime" => self.datetime.as_deref()?,
            "local_datetime" => self.local_datetime.as_deref()?,
            "utc_datetime" => self.utc_datetime.as_deref()?,
            "t" => return Some(self.t.to_string()),
            "hu" => return Some(self.hu.to_string()),
            "tcc" => return Some(self.tcc.to_string()),
            "tp" => return Some(self.tp.to_string()),
            "ws" => return Some(self.ws.to_string()),
            "wd_deg" => return Some(self.wd_deg.to_string()),
            "wd" => self.wd.as_str(),
            "wd_to" => self.wd_to.as_deref()?,
            "vs" => {
                return Some(
                    self.vs
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "null".to_string()),
                )
            }
            "vs_text" => self.vs_text.as_deref()?,
            "weather" => return Some(self.weather.to_string()),
            "weather_desc" => self.weather_desc.as_deref()?,
            "weather_desc_en" => self.weather_desc_en.as_deref()?,
            "image" => self.image.as_deref()?,
            "time_index" => self.time_index.as_deref()?,
            "analysis_date" => self.analysis_date.as_deref()?,
            _ => return None,
        };
        Some(s.to_string())
    }
}

/// Lenient number deserialization (accepts numbers embedded as strings).
fn deserialize_lenient_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(deserializer)?;
    Ok(match v {
        serde_json::Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| D::Error::custom("expected finite f64"))?,
        serde_json::Value::String(s) => s.trim().parse().unwrap_or(0.0),
        _ => 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bmkg_fixture() -> String {
        json!({
            "lokasi": {
                "adm1": "31", "adm2": "31.71", "adm3": "31.71.03",
                "adm4": "31.71.03.1001",
                "provinsi": "DKI Jakarta",
                "kotkab": "Kota Adm. Jakarta Pusat",
                "kecamatan": "Kemayoran",
                "desa": "Kemayoran",
                "lon": 106.849983,
                "lat": -6.17172,
                "timezone": "Asia/Jakarta"
            },
            "data": [
                {
                    "lokasi": {},
                    "cuaca": [
                        [
                            {
                                "datetime": "2026-05-10T06:00:00Z",
                                "t": 31, "hu": 68, "tcc": 39, "tp": 0.2, "ws": 8.7,
                                "wd_deg": 14, "wd": "N", "wd_to": "S",
                                "vs": 8401, "vs_text": "< 9 km",
                                "weather": 1,
                                "weather_desc": "Cerah", "weather_desc_en": "Sunny",
                                "image": "https://api-apps.bmkg.go.id/storage/icon/cuaca/cerah-am.svg",
                                "local_datetime": "2026-05-10 13:00:00",
                                "utc_datetime": "2026-05-10 06:00:00",
                                "analysis_date": "2026-05-10T00:00:00",
                                "time_index": "5-6"
                            },
                            {
                                "local_datetime": "2026-05-10 16:00:00",
                                "weather": 2, "t": 30
                            }
                        ],
                        [
                            {
                                "local_datetime": "2026-05-11 01:00:00",
                                "weather": 60, "t": 27
                            }
                        ]
                    ]
                }
            ]
        })
        .to_string()
    }

    #[test]
    fn parse_bmkg_shape() {
        let f = Forecast::parse(&bmkg_fixture()).expect("parse bmkg");
        assert_eq!(f.location.desa, "Kemayoran");
        assert_eq!(f.location.lat, -6.17172);
        assert_eq!(f.groups.len(), 2);
        assert_eq!(f.slots().count(), 3);
        let s = f.first_slot().expect("slot");
        assert_eq!(s.t, 31);
        assert_eq!(s.desc(Lang::EN), "Sunny");
        assert_eq!(s.desc(Lang::ID), "Cerah");
        assert_eq!(
            s.local_date(),
            Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap())
        );
        assert_eq!(s.local_hour(), Some(13));
        assert_eq!(s.vs_text.as_deref(), Some("< 9 km"));
    }

    #[test]
    fn parse_canonical_shape_roundtrip() {
        let f = Forecast::parse(&bmkg_fixture()).expect("parse");
        let canonical = serde_json::to_string(&f).unwrap();
        let f2 = Forecast::parse(&canonical).expect("parse canonical");
        assert_eq!(f, f2);
    }

    #[test]
    fn parse_empty_is_ok() {
        let f = Forecast::parse("{}").expect("empty");
        assert!(f.is_empty());
    }

    #[test]
    fn parse_garbage_errors() {
        assert!(Forecast::parse("not json").is_err());
    }

    #[test]
    fn slot_value_lookup() {
        let f = Forecast::parse(&bmkg_fixture()).unwrap();
        let s = f.first_slot().unwrap();
        assert_eq!(s.value("t").as_deref(), Some("31"));
        assert_eq!(s.value("weather_desc_en").as_deref(), Some("Sunny"));
        assert_eq!(s.value("wd_deg").as_deref(), Some("14"));
        assert_eq!(s.value("nosuch"), None);
    }

    #[test]
    fn location_parses_string_numbers() {
        let v = json!({ "lat": "-6.17", "lon": "106.8" });
        let loc: Location = serde_json::from_value(v).unwrap();
        assert_eq!(loc.lat, -6.17);
        assert_eq!(loc.lon, 106.8);
    }
}
