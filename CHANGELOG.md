# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-10-17

First release of the rewritten pure-logic library. This replaces the
library-and-binary `cuaca` crate (≤ 0.3.0); the CLI/daemon moved to a separate
`cuaca-cli` repository.

### Added

- Typed forecast models: `Forecast` (`Location`, `SlotGroup`, `Slot`) that
  deserialize directly from raw BMKG JSON as well as a canonical serde-friendly
  form, with `CuacaError::Parse` fallbacks and a `Slot::value` key surface for
  `{KEY}` templating.
- `Config` struct (serde) replacing the clap `Args`, with the old CLI defaults
  (en, bar, `%Y-%m-%d`, yticks 4) provided through `Default`.
- `waybar::render`: Waybar `{text, tooltip, class}` output with pango
  escaping, localized tooltips, `⚠️` warnings prefix, warnings section, CSS
  classes, `error_json`; plus `WaybarOut::to_json` serialization.
- `terminal::render`: full terminal display (header, ASCII art block, 6
  column charts with y-ticks, tomorrow/day-after tables).
- `icons`: emoji / nerd / 5-line ASCII mappings (was core/constants).
- `format`: time/temp/wind formatting + custom indicator engine (was
  core/format).
- `color`: ANSI/pango colorization + `expand_color_tokens` (was core/color),
  with a direct `set_color_enabled` helper.
- `lang`: EN/ID labels (was core/l10n).
- `warnings`: typed `Warning` with polygons, `parse_polygon`,
  `point_in_polygon`, `contains_point`, `matches_province`, and
  `select_relevant` (the polygon→province→validity cascade from the old
  fetch pipeline).
- `stats`: pure `summarize` over `ArchiveEntry` slices + `to_table`/`to_json`
  renderers (ex `cuaca stats` command logic).
- GitHub Actions: `test.yaml` (fmt/clippy/test) and `publish.yaml`
  (crates.io on tag push).
- 92 tests (unit + integration + doctest).

### Changed

- Crate is library-only: no binaries, no CLI, no daemon, no installer
  scripts (moved to the future `cuaca-cli` repo).
- Raw `serde_json::Value` plumbing was removed in favor of typed models.
- `wind_dir_icon` now correctly handles negative degrees (previous -45 
  rounding bug; behavior now wraps like 315).
- Dependency set reduced to `chrono`, `serde`, `serde_json`, `thiserror`
  (dropped clap, reqwest, quick-xml, directories, signal-hook).

### Removed

- `cuaca` binary, `cuaca server`, `cuaca client`, `cuaca stats` commands.
- File cache, location resolution, HTTP fetching, CAP XML parsing (I/O now
  belongs to consumers).
- install.sh/install.ps1 installers and binary-release workflow.
