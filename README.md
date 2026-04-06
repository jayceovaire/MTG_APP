# MTG App

MTG App is a desktop Magic: The Gathering companion built with Tauri, Vue 3, Vuetify, and Rust. It combines collection tracking, deck and package management, card image caching, commander-aware deck editing, and deck analysis tooling in a single local-first application.

The app is designed around a simple idea: keep the responsive UI in Vue, keep stateful and file-backed operations in Rust, and persist user data locally so the app remains fast and usable without requiring an online account or hosted backend.

## What The App Does

MTG App currently focuses on six core workflows:

- Build and maintain Commander decks with a desktop-first editing experience.
- Organize reusable packages of cards that can be applied across multiple decks.
- Track a personal card collection, including favorites and quick-add flows.
- Evaluate decks using CRISPI-based role and power heuristics.
- Run probability and simulation tools against real deck contents.
- Cache Scryfall card images locally so repeated browsing is fast and API-friendly.

The app includes these main views:

- `Home`: recent cached image display and random card surfacing.
- `Decks`: deck library plus a full deck editor.
- `Packages`: reusable package library plus a package editor.
- `Collection`: personal collection management.
- `Tools`: CRISPI analysis, hypergeometric calculations, and Monte Carlo simulation.
- `Roast`: currently a styled placeholder page for a future AI deck roaster.
- `Settings`: updater entry point and bulk image download tools.
- `Information`: app information and supporting content.
- `Card Viewer`: search-driven card lookup view.

## How It Works

The project is split into two layers:

- The frontend in `src/` is a Vue 3 single-page app using Composition API and Vuetify.
- The backend in `src-tauri/` is a Rust Tauri application that exposes commands to the frontend through Tauri IPC.

In practice, the flow looks like this:

1. A Vue view calls a helper in `src/api/`.
2. That helper uses `invoke()` to call a named Rust command.
3. Rust reads or mutates local state, SQLite-backed data, or local files.
4. Rust returns structured data back to the Vue view.
5. Some long-running backend tasks also emit Tauri events so the UI can update in real time.

This architecture keeps UI concerns in the frontend and pushes persistence, filesystem access, SQLite, updater logic, and image caching into the native layer.

## Feature Overview

### Deck Management

Decks are first-class objects in the app. Users can:

- Create, rename, duplicate, and delete decks.
- Add cards individually or through bulk import flows.
- Assign commanders and partners.
- Remove commanders or replace them cleanly.
- Attach reusable packages to decks.
- Inspect deck contents in a persistent local editor.

Deck data is stored as serialized JSON inside the local user database, while the active in-memory copy is held in `AppState` on the Rust side for fast access.

### Package Management

Packages are reusable card groups intended to reduce repetitive editing across multiple decks. A package can hold its own name, description, and card list, then be attached to decks from the deck editor.

This is useful for:

- shared mana packages
- recurring interaction suites
- color-specific staple bundles
- archetype modules
- test packages for theorycrafting

Packages are persisted separately from decks and can be duplicated, edited, and deleted independently.

### Collection Tracking

The collection workflow supports:

- adding cards by name
- bulk adding multiple cards
- duplicating collection entries
- removing cards
- favoriting and unfavoriting cards
- searching card suggestions from the local card database

Collection cards are persisted locally, and favorites are stored in a separate table for quick filtering and retrieval.

### CRISPI Deck Analysis

The Tools view uses CRISPI-oriented evaluation logic from the Rust backend to inspect a deck and produce structured role and power data.

The CRISPI tooling includes:

- role counts across the deck
- card-level role evaluation
- dimension scoring such as consistency, resilience, interaction, speed, and pivotability
- tier or interpretation-oriented output used by the frontend for summary presentation

The frontend then visualizes those results and uses them to drive probability tooling.

### Hypergeometric Calculator

The hypergeometric calculator uses a selected deck and CRISPI output to estimate draw odds. It lets the user work with:

- total population size
- matching cards in the library
- sample size by turn
- target hit counts
- optional card type filtering
- optional CRISPI role filtering

This is useful for questions like:

- What are the odds of seeing at least one ramp card by turn 3?
- How often does this deck see interaction in the opening hand?
- What is the probability of finding a specific role bucket by a given turn?

### Monte Carlo Simulation

The Monte Carlo tool runs repeated randomized draws against a deck to estimate role appearance over time. The current implementation simulates:

- opening hands
- one draw per turn
- turn-by-turn role access across the early game

That makes it useful for checking how often a deck naturally sees enough ramp, removal, or other role categories without relying only on exact closed-form probability formulas.

### Local Image Caching

The app can download and cache card images from Scryfall. This is handled by Rust so the filesystem and network work stays outside the renderer process.

Image caching behavior includes:

- image downloads for collection cards, decks, packages, or everything
- local cache storage under the app data directory
- progress events emitted back to the frontend
- a semaphore/rate-limiting approach intended to stay friendly to Scryfall

The Settings page exposes a bulk image download action, and multiple views listen for image-related events so cached visuals refresh automatically when downloads finish.

### In-App Updating

The app is wired for Tauri updater support. The current implementation includes:

- manual update checks from Settings
- startup update checks that can prompt the user with a modal
- download/install/restart handling via Tauri updater commands

For this to work in a packaged release, the updater configuration, release artifacts, manifest JSON, and signing metadata all need to be consistent.

## Desktop Architecture

### Frontend

The Vue side is responsible for:

- routing
- screen layout
- forms and controls
- local view state
- progress and status display
- rendering deck, package, collection, and analysis screens

Key frontend folders:

- `src/views/`: page-level screens
- `src/components/`: reusable UI components
- `src/api/`: thin wrappers around Tauri `invoke()` calls
- `src/router/`: route definitions
- `src/utils/`: small frontend utility helpers

### Backend

The Rust side is responsible for:

- app startup and plugin registration
- SQLite-backed persistence
- deck/package/collection mutation logic
- card search and retrieval
- image downloading and caching
- CRISPI deck evaluation
- updater checks and installation

Key backend files and folders:

- `src-tauri/src/lib.rs`: Tauri app setup, plugin wiring, command registration
- `src-tauri/src/state.rs`: application state and persistence bootstrap
- `src-tauri/src/commands/`: command handlers grouped by domain
- `src-tauri/src/models/`: Rust data structures for cards, decks, packages, and analysis

## Data Model And Persistence

The app is local-first. There is no hosted sync layer in this repository.

On startup, Rust creates or opens a SQLite database in the platform-local data directory, then loads its data into memory.

Current persisted tables include:

- `collection_cards`
- `favorites`
- `packages`
- `decks`

The user database lives under the local app data directory in:

- Windows: `%LOCALAPPDATA%\mtg_app\user_data.db`
- macOS: `~/Library/Application Support/mtg_app/user_data.db`
- Linux: `~/.local/share/mtg_app/user_data.db`

Card images are cached separately under the same app data root in a `card_images` directory.

The bundled card metadata database is included as an app resource:

- `src/db/scryfall.db`

That database is used to support card lookups and related card-oriented workflows inside the app.

## Tech Stack

- `Tauri 2` for the desktop shell and native command bridge
- `Vue 3` with Composition API for the frontend
- `Vuetify` for UI components
- `Rust` for native application logic
- `rusqlite` for local persistence
- `reqwest` and `tokio` for async network and file work
- `Vite` for frontend builds
- `Bun` for frontend package management and scripts

## Requirements

Before running the project locally, install:

- `Bun`
- `Rust` and `cargo`
- Tauri platform prerequisites for your OS

Tauri prerequisites vary by platform. Use the official Tauri documentation to ensure WebView/runtime dependencies are installed for your environment.

## Getting Started

### 1. Clone The Repository

```bash
git clone <repository-url>
cd MTG_APP
```

### 2. Install Frontend Dependencies

```bash
bun install
```

### 3. Run The App In Development

```bash
bun run tauri dev
```

This starts the Vite dev server and launches the Tauri desktop window.

### 4. Build A Production Release

```bash
bun run tauri build
```

This produces packaged desktop artifacts under `src-tauri/target/release/bundle/`.

## Available Scripts

| Script | Purpose |
| --- | --- |
| `bun run dev` | Start the Vite frontend dev server. |
| `bun run build` | Build the Vue frontend for production. |
| `bun run preview` | Preview the production frontend build. |
| `bun run tauri dev` | Run the desktop app in development mode. |
| `bun run tauri build` | Build packaged desktop artifacts. |

## Project Layout

```text
MTG_APP/
|-- src/
|   |-- api/
|   |-- assets/
|   |-- components/
|   |-- router/
|   |-- utils/
|   |-- views/
|   |-- App.vue
|   `-- main.js
|-- src-tauri/
|   |-- capabilities/
|   |-- icons/
|   |-- src/
|   |   |-- commands/
|   |   |-- models/
|   |   |-- lib.rs
|   |   |-- main.rs
|   |   `-- state.rs
|   |-- Cargo.toml
|   `-- tauri.conf.json
|-- dist/
|-- package.json
|-- bun.lock
`-- README.md
```

## Release And Updater Notes

The app is configured to create updater artifacts, but a working release flow requires all of the following to line up:

- the app version in `package.json`
- the app version in `src-tauri/Cargo.toml`
- the Tauri version in `src-tauri/tauri.conf.json`
- the built installer artifact for that version
- the updater signature for that exact artifact
- the `latest.json` manifest used by Tauri updater

In other words, if you publish `0.1.2`, the installer URL and signature in `latest.json` must both describe the `0.1.2` installer, not an older file.

The updater manifest also requires `pub_date` in full RFC 3339 format, for example:

```json
"pub_date": "2026-04-06T00:00:00Z"
```

For manual first-time installs from GitHub, users only need the installer artifact such as the `.msi`. They do not need to separately download the `.sig` file.

## Notes On Current Scope

Some features are more complete than others.

- Deck, package, collection, image caching, and CRISPI workflows are implemented and wired into the desktop app.
- The Roast page is currently a placeholder for a future AI-assisted deck roasting experience.
- Automated frontend tests are not yet present in this repository.

## Verification

Useful local verification commands:

```bash
bun run build
cd src-tauri
cargo check
```

## Documentation References

Additional internal documentation lives in:

- `src-tauri/Guidelines/CRISPI.md`
- `src-tauri/Guidelines/MTG_Role_Inference_Engine_Agent_Spec.md`

Those files describe the deck evaluation framework and role inference logic in more detail.

## License

No license file is currently included in this repository.
