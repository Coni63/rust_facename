# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build            # debug build
cargo build --release  # optimized build
cargo run              # build and run
cargo check            # fast type check
cargo clippy           # lint
cargo fmt              # format code
cargo test             # run tests (none currently exist)
```

## Architecture

**Facename** is a desktop GUI quiz app (egui/eframe) that helps users memorize faces, names, and job positions. The UI is in French.

### State Machine

The app flows through four states defined in `AppState` (game_state.rs):
- `Menu` → `Learning` → `Question` → `Results`

`Learning` auto-advances slides on a timer (default 5s per person). `Question` and `Results` are stubs not yet implemented.

### Key files

- **`src/main.rs`** — initializes eframe window (400×300) and installs image loaders via `egui_extras`
- **`src/game_state.rs`** — all data structs (`GameData`, `Sample`, `GameSettings`, `MyGameApp`) and `create_dataset()` which randomly pairs images with names/positions from config
- **`src/ui.rs`** — implements `eframe::App`; renders each state via `show_menu()`, `show_learning()`, `show_questions()`, `show_results()`

### Data flow

`assets/config.json` is loaded at startup and deserialized into `GameData`. It contains male/female name pools, last names, positions, and image metadata (filename → gender). `create_dataset()` shuffles images, picks N based on settings, then assigns random names respecting gender.

Images are loaded by egui using `file://` URIs pointing into `assets/img/`.

### Modes (defined, not yet integrated)

`Mode` enum defines four quiz directions: `FaceToName`, `NameToFace`, `FaceToPosition`, `PositionToFace` — only the data structures exist; no mode-specific UI or answer validation is implemented yet.
