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

`AppState` (game_state.rs): `Menu` → `Learning` → `Question` → `Results`. The menu has one button per game (6 `Mode`s) plus a `Difficulty` selector. `Learning` shows N profiles (photo, name, job) on a timer; `Question` asks K <= N questions (4 choices, distractors mix already-seen profiles and invented values) with immediate feedback.

### Key files

- **`src/main.rs`** — eframe window and image loaders (`egui_extras`)
- **`src/modes.rs`** — `Attr` (Face/Name/Job), `Mode` (cue → answer), `Difficulty` (N, K, learning time), `GameSettings`
- **`src/data.rs`** — `GameData::load()`: `Config` from `assets/config.json` (names, last names, jobs) + `Face`s scanned from `assets/img/men/` and `assets/img/women/` (sex = folder)
- **`src/game_state.rs`** — `AppState`, `Sample`, `QuizQuestion`, `MyGameApp` and game generation (`start_game`: dataset with unique faces/names/jobs, then questions)
- **`src/ui.rs`** — `eframe::App` impl; `show_menu/learning/questions/results`

Images are loaded by egui via `file://assets/img/<men|women>/<file>` URIs. Number of profiles is capped by available faces/jobs.

### Data

Images comes from: https://uifaces.co/
Dedup check (2026-11): SHA-256 over all images in `men/` + `women/` found no exact duplicates (44 men, 91 women). Re-run after adding scraped images: `cd assets/img && sha256sum men/* women/* | awk '{print $1}' | sort | uniq -d`. Near-duplicates (resized/re-encoded) are not detected.

### Conventions & decisions

- UI text stays in French; names, last names and jobs in `config.json` are French.
- Names/jobs are randomly assigned per game, so a given photo gets a different identity each time.
- Face-answer games draw distractors from the whole face pool, so choices are fewer than 4 only if there are too few images.
- Store any further notes about the code here in CLAUDE.md (user preference).
- Distractors for Name/Face answers are always the same sex as the correct answer (seen profiles and invented ones alike); Job distractors are not filtered (job titles in config.json are mixed masculine/feminine, e.g. "Directrice Commerciale" vs "Directeur Financier", and are assigned regardless of sex).
