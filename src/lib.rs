//! # cuaca
//!
//! Pure-logic core of the cuaca weather toolchain: typed models for BMKG
//! forecast data, Waybar and terminal rendering, weather icons, EN/ID
//! localization, and forecast statistics.
//!
//! This crate is **I/O free**: no HTTP, no filesystem, no CLI. Consumers such
//! as the `cuaca-cli` binary (fetching + caching + daemon) and the
//! `cuaca-worker` service do their own networking and feed the results in.
//!
//! ## Quick start
//!
//! ```no_run
//! use cuaca::{Config, Forecast, waybar};
//!
//! // A raw BMKG JSON payload (from cuaca-worker's /api/forecast or the BMKG API).
//! let raw = std::fs::read_to_string("forecast.json")
//!     .map_err(|e| cuaca::CuacaError::Unknown(e.to_string()))?;
//! let forecast = Forecast::parse(&raw)?;
//!
//! let cfg = Config::default();
//! let out = waybar::render(&cfg, &forecast, &[])?;
//! println!("{}", out.to_json()?);
//!
//! // Or a plain terminal display:
//! print!("{}", cuaca::terminal::render(&cfg, &forecast, &[]));
//! # Ok::<(), cuaca::CuacaError>(())
//! ```
//!
//! ## Deserializing forecasts
//!
//! [`Forecast`] accepts both the raw BMKG shape (t, hu, tcc, tp, ws, wd_deg,
//! weather, …) and this crate's canonical serialized shape, so archives can
//! round-trip through `serde`.
//!
//! ## Feature map
//!
//! | Module       | Purpose                                            |
//! |--------------|---------------------------------------------------|
//! | [`config`]   | [`Config`] + [`OutputFormat`]                      |
//! | [`model`]    | [`Forecast`], [`Location`], [`SlotGroup`], [`Slot`]|
//! | [`waybar`]   | `{ text, tooltip, class }` rendering               |
//! | [`terminal`] | multi-line terminal display                        |
//! | [`graphs`]   | column-chart panels                                |
//! | [`icons`]    | emoji / nerd / ASCII icon tables                   |
//! | [`format`]   | formatting helpers + `{KEY}` indicator templates   |
//! | [`color`]    | ANSI and Pango colorization                        |
//! | [`lang`]     | EN/ID localization                                 |
//! | [`warnings`] | typed warnings + polygon matching                  |
//! | [`stats`]    | forecast archive statistics                        |

pub mod color;
pub mod config;
pub mod error;
pub mod format;
pub mod graphs;
pub mod icons;
pub mod lang;
pub mod model;
pub mod stats;
pub mod terminal;
pub mod util;
pub mod warnings;
pub mod waybar;

pub use color::ColorMode;
pub use config::{Config, OutputFormat};
pub use error::CuacaError;
pub use format::{celsius_to_fahrenheit, format_indicator, format_temp, format_time};
pub use icons::{
    ascii_icon, weather_icon, wind_dir_icon, CLOUD_COVER_ICON, ERROR_ICON, ICON_PLACEHOLDER,
    PRECIPITATION_ICON, VISIBILITY_ICON,
};
pub use lang::Lang;
pub use model::{Forecast, Location, Slot, SlotGroup};
pub use util::error_json;
pub use warnings::Warning;
pub use waybar::WaybarOut;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_reexports_working() {
        color::set_color_enabled(false);
        let raw = r#"{
          "lokasi": { "desa": "Kemayoran", "provinsi": "DKI Jakarta",
            "lat": -6.17, "lon": 106.84 },
          "data": [ { "cuaca": [
            [ { "local_datetime": "2026-05-10 13:00:00", "weather": 1,
                "weather_desc": "Cerah", "weather_desc_en": "Sunny", "t": 31 } ]
          ] } ]
        }"#;
        let f = Forecast::parse(raw).unwrap();
        let cfg = Config::default();
        let out = waybar::render(&cfg, &f, &[]).unwrap();
        assert!(out.text.contains("31\u{00b0}"));
        assert_eq!(out.class, "sunny");
        assert_eq!(f.location.desa, "Kemayoran");
    }
}
