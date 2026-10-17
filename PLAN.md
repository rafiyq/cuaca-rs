# cuaca-rs

The **`cuaca`** library (crates.io): pure-logic core of the cuaca weather toolchain.

## Project Status

v1.0.0 — library-only rewrite of `cuaca` (the Rust Waybar/R weather CLI).

## Architecture

```
cuaca-rs (this repo, pure logic, no I/O)
        ▲                    ▲
        │                    │
cuaca-worker (TS)      cuaca-cli (future repo)
 fetch + render via     fetch + cache + daemon + CLI
 JSON; own I/O          parses BMKG JSON → Forecast
```

- **This crate**: typed models (`Forecast`, `Location`, `SlotGroup`, `Slot`), Waybar + terminal renderers, icon tables (emoji / nerd / ASCII), EN/ID localization, `{KEY}` indicator templating, ANSI/Pango colorization, polygon-based warning matching, forecast statistics.
- **I/O lives in consumers**: HTTP fetching (BMKG + location lookup), caching, cli parsing, daemon sockets.

## Source Files

| File | Purpose |
|------|---------|
| `src/model.rs` | Typed entities; parses raw BMKG payload AND canonical form |
| `src/config.rs` | `Config` + `OutputFormat` (serde; defaults from the old CLI) |
| `src/waybar.rs` | `{ text, tooltip, class }` renderer (ex `cli/run.rs`) |
| `src/terminal.rs` | Terminal renderer (ex `display/terminal`) |
| `src/graphs.rs` | Column chart panels (ex `core/graphs`) |
| `src/icons.rs` | Emoji / nerd / ASCII tables (ex `core/constants`) |
| `src/format.rs` | Formatting helpers + indicator engine (ex `core/format`) |
| `src/color.rs` | Colorization + Pango conversion (ex `core/color`) |
| `src/lang.rs` | EN/ID localization (ex `core/l10n`) |
| `src/warnings.rs` | Typed warnings + polygon matching (ex `core/warnings/{polygon}`) |
| `src/stats.rs` | Pure stats (ex `stats.rs` minus stdout/printing) |
| `src/util.rs` | `escape_pango`, `error_json` |

Behavioral spec came from rafiyq/cuaca at v0.3.0; logic was re-ported with identical output paths (tooltip formatting, class names, table layouts, polygon cascade).

## Key Design Decisions

- **Zero I/O**: no network/cache/filesystem in the library.
- **Typed data**: raw `serde_json::Value` threading replaced by serde structs.
- **Serde field names mirror BMKG** so consumers deserialize the raw API response directly.
- **Config serde first-class**: doubles as a daemon wire protocol (like the old `Args`).
- **No clap**: CLI shape moves to the future `cuaca-cli` repo.
- **Deps kept minimal**: chrono, serde, serde_json, thiserror.
- **License:** MIT; BMKG attribution in tooltip and README.

## Publishing

- **GitHub:** https://github.com/rafiyq/cuaca-rs
- **crates.io:** published as `cuaca` on `v*` tags (`.github/workflows/publish.yaml`).

## Roadmap

- [x] Typed model + BMKG/canonical dual shape
- [x] Waybar + terminal renderers ported (parity tests)
- [x] Icons, graphs, color, lang, format ports
- [x] Warnings polygon matching port
- [x] Stats as pure functions
- [ ] `cuaca-cli` repo (fetch/cache/daemon on top of this lib)
- [ ] `cuaca-worker` TS types mirrored from `model.rs`

See `CHANGELOG.md` for version history.
