//! Localization strings for tooltips and terminal output (EN/ID).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lang {
    #[default]
    EN,
    ID,
}

impl Lang {
    pub fn humidity(&self) -> &'static str {
        match self {
            Self::EN => "Humidity",
            Self::ID => "Kelembaban",
        }
    }

    pub fn cloud_cover(&self) -> &'static str {
        match self {
            Self::EN => "Cloud Cover",
            Self::ID => "Tutupan Awan",
        }
    }

    pub fn precipitation(&self) -> &'static str {
        match self {
            Self::EN => "Precipitation",
            Self::ID => "Curah Hujan",
        }
    }

    pub fn wind(&self) -> &'static str {
        match self {
            Self::EN => "Wind",
            Self::ID => "Angin",
        }
    }

    pub fn visibility(&self) -> &'static str {
        match self {
            Self::EN => "Visibility",
            Self::ID => "Jarak Pandang",
        }
    }

    pub fn location(&self) -> &'static str {
        match self {
            Self::EN => "Location",
            Self::ID => "Lokasi",
        }
    }

    pub fn source(&self) -> &'static str {
        match self {
            Self::EN => "Source: BMKG (Badan Meteorologi, Klimatologi, dan Geofisika)",
            Self::ID => "Sumber: BMKG (Badan Meteorologi, Klimatologi, dan Geofisika)",
        }
    }

    pub fn today(&self) -> &'static str {
        match self {
            Self::EN => "Today",
            Self::ID => "Hari Ini",
        }
    }

    pub fn tomorrow(&self) -> &'static str {
        match self {
            Self::EN => "Tomorrow",
            Self::ID => "Besok",
        }
    }

    pub fn day_after_tomorrow(&self) -> &'static str {
        match self {
            Self::EN => "Day After Tomorrow",
            Self::ID => "Lusa",
        }
    }

    pub fn weather_desc_key(&self) -> &'static str {
        match self {
            Self::EN => "weather_desc_en",
            Self::ID => "weather_desc",
        }
    }

    pub fn temperature(&self) -> &'static str {
        match self {
            Self::EN => "TEMPERATURE",
            Self::ID => "SUHU",
        }
    }

    pub fn rainfall(&self) -> &'static str {
        match self {
            Self::EN => "RAINFALL (mm)",
            Self::ID => "HUJAN (mm)",
        }
    }

    pub fn humidity_label(&self) -> &'static str {
        match self {
            Self::EN => "HUMIDITY (%)",
            Self::ID => "KELEMBABAN (%)",
        }
    }

    pub fn wind_label(&self) -> &'static str {
        match self {
            Self::EN => "WIND (km/h)",
            Self::ID => "ANGIN (km/j)",
        }
    }

    pub fn cloud_label(&self) -> &'static str {
        match self {
            Self::EN => "CLOUD COVER (%)",
            Self::ID => "AWAN (%)",
        }
    }

    pub fn visibility_label(&self) -> &'static str {
        match self {
            Self::EN => "VISIBILITY (km)",
            Self::ID => "JARAK PANDANG (km)",
        }
    }

    pub fn temperature_base(&self) -> &'static str {
        match self {
            Self::EN => "TEMPERATURE",
            Self::ID => "SUHU",
        }
    }

    pub fn total(&self) -> &'static str {
        match self {
            Self::EN => "Total",
            Self::ID => "Jumlah",
        }
    }

    pub fn weather_report(&self) -> &'static str {
        match self {
            Self::EN => "Weather Report",
            Self::ID => "Prakiraan Cuaca",
        }
    }

    pub fn wind_unit(&self) -> &'static str {
        match self {
            Self::EN => "km/h",
            Self::ID => "km/j",
        }
    }

    pub fn average_label(&self) -> &'static str {
        match self {
            Self::EN => "avg",
            Self::ID => "rata-rata",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_en() {
        assert_eq!(Lang::EN.humidity(), "Humidity");
        assert_eq!(Lang::EN.today(), "Today");
        assert_eq!(Lang::EN.weather_desc_key(), "weather_desc_en");
        assert_eq!(Lang::EN.wind_unit(), "km/h");
    }

    #[test]
    fn labels_id() {
        assert_eq!(Lang::ID.humidity(), "Kelembaban");
        assert_eq!(Lang::ID.today(), "Hari Ini");
        assert_eq!(Lang::ID.weather_desc_key(), "weather_desc");
        assert_eq!(Lang::ID.wind_unit(), "km/j");
        assert_eq!(Lang::ID.day_after_tomorrow(), "Lusa");
    }

    #[test]
    fn default_is_en() {
        assert_eq!(Lang::default(), Lang::EN);
    }
}
