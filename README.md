# cuaca

[![crates.io](https://img.shields.io/crates/v/cuaca)](https://crates.io/crates/cuaca)
[![docs.rs](https://img.shields.io/docsrs/cuaca)](https://docs.rs/cuaca)
[![CI](https://github.com/rafiyq/cuaca-rs/actions/workflows/test.yaml/badge.svg?branch=main)](https://github.com/rafiyq/cuaca-rs/actions/workflows/test.yaml)
[![License: MIT](https://img.shields.io/crates/l/MIT)](https://github.com/rafiyq/cuaca-rs/blob/main/LICENSE)

Pure-logic core of the **cuaca** weather toolchain for [BMKG](https://www.bmkg.go.id/) (Indonesian weather agency) data — typed forecast models, [Waybar](https://github.com/Alexays/Waybar) and terminal rendering, icons, EN/ID localization, and forecast statistics.

This crate does **no I/O**: no HTTP, no filesystem, no CLI binaries. Consumers do their own fetching:

- `cuaca-cli` (separate repo) — the previous `cuaca` binary, rebuilt on top of this library
- `cuaca-worker` (separate repo) — the wttr.in-style Cloudflare Worker service

## Install

```bash
cargo add cuaca
```

or in `Cargo.toml`:

```toml
[dependencies]
cuaca = "1"
```

## Quick start (Waybar)

Feed raw BMKG JSON (from the [BMKG public API](https://data.bmkg.go.id/prakiraan-cuaca/) or your own service) into a `Forecast` and render:

```rust
use cuaca::{Config, Forecast, waybar};

let raw = /* raw BMKG forecast JSON */;

let forecast = Forecast::parse(raw)?;
let cfg = Config::default();
let out = waybar::render(&cfg, &forecast, &[/* cuaca::Warning */])?;
println!("{}", out.to_json()?); // {"text":..,"tooltip":..,"class":..}
# Ok::<(), cuaca::CuacaError>(())
```

Terminal display:

```rust
use cuaca::{Config, Forecast, terminal};

let forecast = Forecast::parse(raw)?;
print!("{}", terminal::render(&Config::default(), &forecast, &[]));
```

Custom bar text uses `{KEY}` placeholders (`{t}`, `{hu}`, `{weather_desc_en}`, `{wd_deg}`, …):

```rust
use cuaca::{Config, Forecast, waybar};

let cfg = Config {
    custom_indicator: Some("{ICON} {t}°C".to_string()),
    ..Config::default()
};
let out = waybar::render(&cfg, &Forecast::parse(raw)?, &[])?;
# Ok::<(), cuaca::CuacaError>(())
```

## What the library provides

| Module | Purpose |
|--------|---------|
| `model` | `Forecast` / `Location` / `SlotGroup` / `Slot` typed entities |
| `config` | `Config` + `OutputFormat` (serde-friendly) |
| `waybar` | `{ text, tooltip, class }` rendering + `error_json` |
| `terminal` | multi-line terminal renderer with charts |
| `graphs` | column-chart panels with y-ticks |
| `icons` | emoji / nerd font glyphs / 5-line ASCII art per BMKG code |
| `format` | time / temperature / wind formatting + `{KEY}` indicator engine |
| `color` | ANSI + Pango colorization (`ColorMode::Auto/Always/Never`) |
| `lang` | EN/ID localization strings |
| `warnings` | typed weather warnings + polygon containment matching |
| `stats` | pure aggregation over archived forecasts |

`Forecast` deserializes **both** the raw BMKG response shape (`lokasi`/`data[].cuaca`) and this crate's canonical serialized shape, so data can be archived and round-tripped losslessly.

## License

MIT — weather data © [BMKG](https://www.bmkg.go.id).
