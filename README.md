# MTG App (Tauri + Vue 3)

A desktop application for Magic: The Gathering card collection and deck building, built with [Tauri](https://tauri.app/), [Vue 3](https://vuejs.org/), and [Rust](https://www.rust-lang.org/).

## 🌟 Key Features

- **Advanced Deck Management:** Full-featured deck builder with "Import Decklist" capabilities (Quantity CardName format).
- **CRISPI v2 Power Evaluation:** A sophisticated power-level scoring framework (Consistency, Resilience, Interaction, Speed, Pivotability) that uses:
  - Global efficiency rules based on mana value (MV).
  - Non-linear AMV multipliers for high-curve penalties.
  - Role-based classification (Protection, Recursion, Engines, Wincons, etc.) with strict logic for utility lands.
  - Integration with "Game Changers" (high-impact cards) for accurate bracket-based tiering.
- **Monte Carlo & Hypergeometric Simulations:** Real-time probability calculations for drawing specific card roles across turn-by-turn simulations and opening hand scenarios.
- **Polished UI:** User-friendly role names (e.g., "FAST MANA" instead of "FAST_MANA") and dynamic role distribution charts.
- **Targeted Image Caching:** 
  - Automated, background image fetching for specific decks, packages, or individual card searches.
  - Scryfall API compliance with a global 100ms rate limiter and 10 requests-per-second cap.
  - Real-time UI updates via Tauri event emitters (`images-updated`).
- **Expanded Commander Rules:** Support for Legendary Vehicles and Legendary Spacecraft as valid commanders.
- **Collection Tracker:** Search and manage your personal card collection with suggestion-driven UI.

## 🛠 Tech Stack

- **Frontend Framework:** [Vue 3](https://vuejs.org/) (Composition API with `<script setup>`)
- **UI Component Library:** [Vuetify 3](https://vuetifyjs.com/)
- **State Management & Backend:** [Rust](https://www.rust-lang.org/) (via Tauri)
- **Database:** [SQLite](https://sqlite.org/) (using `rusqlite` on Rust side)
- **Styling:** [Vuetify](https://vuetifyjs.com/), [Mana Font](https://andrewgioia.github.io/Mana/) (MTG symbols)
- **Build Tool:** [Vite](https://vitejs.dev/)
- **Package Manager:** [Bun](https://bun.sh/) (frontend), [Cargo](https://doc.rust-lang.org/cargo/) (backend)

## 📋 Requirements

Before starting, ensure you have the following installed:

- [Node.js](https://nodejs.org/) or [Bun](https://bun.sh/)
- [Rust & Cargo](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://tauri.app/v2/reference/cli/): `cargo install tauri-cli`
- System-specific Tauri dependencies (see [Tauri's prerequisites](https://tauri.app/v2/guides/prerequisites/))

## 🚀 Setup & Run

1.  **Clone the repository:**
    ```bash
    git clone <repository-url>
    cd MTG_APP
    ```

2.  **Install dependencies:**
    Using Bun:
    ```bash
    bun install
    ```
    (Or `npm install` if using npm).

3.  **Run in development mode:**
    ```bash
    bun tauri dev
    ```
    This starts the Vite dev server and launches the Tauri window.

4.  **Build for production:**
    ```bash
    bun tauri build
    ```

## 📜 Available Scripts

| Script | Description |
| :--- | :--- |
| `dev` | Runs the Vite dev server for the frontend. |
| `build` | Builds the frontend for production. |
| `preview` | Previews the production build of the frontend. |
| `tauri` | Accesses the Tauri CLI (e.g., `bun run tauri dev`). |

## 📁 Project Structure

```text
MTG_APP/
├── src/                # Frontend (Vue 3)
│   ├── api/            # API/Type definitions
│   ├── assets/         # Styles, images
│   ├── components/     # Vue components
│   ├── router/         # Vue Router configuration
│   ├── utils/          # Frontend utility functions
│   └── views/          # Page-level Vue components
├── src-tauri/          # Backend (Rust + Tauri)
│   ├── src/            # Rust source code
│   │   ├── commands/   # Tauri command handlers (collection, deck, image, etc.)
│   │   ├── models/     # Rust data models (card, deck, package, crispi)
│   │   ├── lib.rs      # Main Tauri application logic
│   │   ├── state.rs    # AppState and Persistence logic
│   │   └── main.rs     # Binary entry point
│   └── Cargo.toml      # Rust dependencies
├── public/             # Static assets
├── scryfall.db         # Scryfall card database (SQLite)
├── vite.config.js      # Vite configuration
└── package.json        # Frontend dependencies & scripts
```

## ⚙️ Environment Variables

- `TAURI_DEV_HOST`: Used in `vite.config.js` to set the host for development.

## 💾 Data Persistence

The application uses SQLite for data storage:
- **Card Metadata:** Reads from `scryfall.db` (in the root or project-specified path).
- **User Data:** Collections, decks, and favorites are stored in a local SQLite database located in the platform-specific app data directory:
  - Windows: `%LOCALAPPDATA%\mtg_app\user_data.db`
  - Linux: `~/.local/share/mtg_app/user_data.db`
  - macOS: `~/Library/Application Support/mtg_app/user_data.db`
- **Image Cache:** Card images are locally cached in the application data directory to reduce Scryfall API load.

## 📖 Technical Guidelines

For more detailed information on internal frameworks, refer to:
- [`CRISPI.md`](src-tauri/Guidelines/CRISPI.md) - Documentation on the power evaluation algorithm and scoring rules.
- [`MTG_Role_Inference_Engine_Agent_Spec.md`](src-tauri/Guidelines/MTG_Role_Inference_Engine_Agent_Spec.md) - Specification for the card classification system.

## 🧪 Tests

- **Rust Backend:** Unit tests for `crispi_model` (role inference, archetypes, and land engine logic) are implemented and passing.
  - Run via: `cd src-tauri ; cargo test`
- **Vue Frontend:** TODO: Add automated component/view tests.

## 📄 License

- TODO: Add license information.
