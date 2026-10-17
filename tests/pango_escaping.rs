use cuaca::util::escape_pango;

#[test]
fn test_escape_pango() {
    assert_eq!(escape_pango("Hello & World"), "Hello &amp; World");
    assert_eq!(escape_pango("3 < 4 > 2"), "3 &lt; 4 &gt; 2");
    assert_eq!(escape_pango("a & b < c > d"), "a &amp; b &lt; c &gt; d");
    assert_eq!(escape_pango("no special"), "no special");
}

#[test]
fn test_forecast_parse_and_render_end_to_end() {
    let raw = r#"{
      "lokasi": { "desa": "Kemayoran", "kecamatan": "Kemayoran",
        "kotkab": "Kota Adm. Jakarta Pusat", "provinsi": "DKI Jakarta",
        "adm4": "31.71.03.1001", "lat": -6.17, "lon": 106.85 },
      "data": [ { "cuaca": [
        [ { "local_datetime": "2026-05-10 13:00:00", "utc_datetime": "2026-05-10 06:00:00",
            "weather": 1, "weather_desc": "Cerah", "weather_desc_en": "Sunny",
            "t": 31, "hu": 68, "tcc": 39, "tp": 0.2, "ws": 8.7,
            "wd_deg": 14, "wd": "N", "vs": 8401, "vs_text": "< 9 km" } ]
      ] } ]
    }"#;

    let forecast = cuaca::Forecast::parse(raw).expect("parse bmkg");
    assert_eq!(forecast.location.adm4, "31.71.03.1001");

    let cfg = cuaca::Config::default();
    let out = cuaca::waybar::render(&cfg, &forecast, &[]).expect("render");
    assert!(out.text.contains("31\u{00b0}"));
    assert_eq!(out.class, "sunny");
    assert!(out.tooltip.contains("&lt; 9 km"));

    let json = out.to_json().expect("to json");
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid waybar json");
    assert!(v["text"].as_str().unwrap().contains("31°"));
    assert!(v["tooltip"].as_str().unwrap().contains("Weather Report:"));
    assert!(v["tooltip"].as_str().unwrap().contains("&lt; 9 km"));
    assert!(v["tooltip"].as_str().unwrap().contains("Today"));
    assert!(v["tooltip"].as_str().unwrap().contains("Source: BMKG"));
    assert_eq!(v["class"].as_str().unwrap(), "sunny");
}
