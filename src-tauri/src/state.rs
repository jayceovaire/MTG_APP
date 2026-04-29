use crate::models::card_model::Card;
use crate::models::deck_model::Deck;
use crate::models::package_model::Package;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::sync::RwLock;
use tauri::Manager;

#[derive(Default)]
pub struct AppState {
    pub decks: RwLock<Vec<Deck>>,
    pub collection: RwLock<Vec<Card>>,
    pub favorites: RwLock<Vec<u64>>,
    pub packages: RwLock<Vec<Package>>,
    pub sidecar_child: RwLock<Option<tauri_plugin_shell::process::CommandChild>>,
    next_deck_id: AtomicU64,
    next_card_id: AtomicU64,
    next_package_id: AtomicU64,
    user_db_path: OnceLock<PathBuf>,
}

impl AppState {
    pub fn initialize_persistence(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let app_data_dir = app
            .path()
            .local_data_dir()
            .map_err(|e| format!("Failed to resolve local app data directory: {e}"))?
            .join("Anura");
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {e}"))?;

        let db_path = app_data_dir.join("user_data.db");
        let _ = self.user_db_path.set(db_path.clone());
        let connection =
            Connection::open(&db_path).map_err(|e| format!("Failed to open user database: {e}"))?;

        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS collection_cards (
                    card_id INTEGER PRIMARY KEY,
                    card_json TEXT NOT NULL
                 );

                 CREATE TABLE IF NOT EXISTS favorites (
                    card_id INTEGER PRIMARY KEY
                 );

                 CREATE TABLE IF NOT EXISTS packages (
                    package_id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    package_json TEXT NOT NULL
                 );

                 CREATE TABLE IF NOT EXISTS decks (
                    deck_id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    deck_json TEXT NOT NULL
                 );",
            )
            .map_err(|e| format!("Failed to create user database schema: {e}"))?;

        let mut loaded_cards = Vec::new();
        let mut card_stmt = connection
            .prepare("SELECT card_json FROM collection_cards ORDER BY card_id")
            .map_err(|e| format!("Failed to prepare collection load query: {e}"))?;
        let card_rows = card_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to read collection rows: {e}"))?;
        for row in card_rows {
            let card_json = row.map_err(|e| format!("Failed to parse collection row: {e}"))?;
            let card: Card = serde_json::from_str(&card_json)
                .map_err(|e| format!("Failed to deserialize collection card: {e}"))?;
            loaded_cards.push(card);
        }

        let mut loaded_favorites = Vec::new();
        let mut favorite_stmt = connection
            .prepare("SELECT card_id FROM favorites ORDER BY card_id")
            .map_err(|e| format!("Failed to prepare favorites load query: {e}"))?;
        let favorite_rows = favorite_stmt
            .query_map([], |row| row.get::<_, u64>(0))
            .map_err(|e| format!("Failed to read favorite rows: {e}"))?;
        for row in favorite_rows {
            loaded_favorites.push(row.map_err(|e| format!("Failed to parse favorite row: {e}"))?);
        }

        let mut loaded_packages = Vec::new();
        let mut package_stmt = connection
            .prepare("SELECT package_json FROM packages ORDER BY package_id")
            .map_err(|e| format!("Failed to prepare package load query: {e}"))?;
        let package_rows = package_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to read package rows: {e}"))?;
        for row in package_rows {
            let package_json = row.map_err(|e| format!("Failed to parse package row: {e}"))?;
            let package: Package = serde_json::from_str(&package_json)
                .map_err(|e| format!("Failed to deserialize package: {e}"))?;
            loaded_packages.push(package);
        }
        loaded_packages.sort_by_key(|p| p.ui_order());

        let mut loaded_decks = Vec::new();
        let mut deck_stmt = connection
            .prepare("SELECT deck_json FROM decks ORDER BY deck_id")
            .map_err(|e| format!("Failed to prepare deck load query: {e}"))?;
        let deck_rows = deck_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to read deck rows: {e}"))?;
        for row in deck_rows {
            let deck_json = row.map_err(|e| format!("Failed to parse deck row: {e}"))?;
            let mut deck: Deck = serde_json::from_str(&deck_json)
                .map_err(|e| format!("Failed to deserialize deck: {e}"))?;
            deck.recount_game_changers();
            loaded_decks.push(deck);
        }
        loaded_decks.sort_by_key(|d| d.ui_order());

        let max_card_id = loaded_cards.iter().map(Card::id).max().unwrap_or(0);
        let max_deck_id = loaded_decks.iter().map(Deck::id).max().unwrap_or(0);
        let max_package_id = loaded_packages.iter().map(Package::id).max().unwrap_or(0);

        {
            let mut collection = self
                .collection
                .write()
                .map_err(|_| "Failed to lock collection during initialization".to_string())?;
            *collection = loaded_cards;
        }
        {
            let mut decks = self
                .decks
                .write()
                .map_err(|_| "Failed to lock decks during initialization".to_string())?;
            *decks = loaded_decks;
        }
        {
            let mut favorites = self
                .favorites
                .write()
                .map_err(|_| "Failed to lock favorites during initialization".to_string())?;
            *favorites = loaded_favorites;
        }
        {
            let mut packages = self
                .packages
                .write()
                .map_err(|_| "Failed to lock packages during initialization".to_string())?;
            *packages = loaded_packages;
        }

        self.next_card_id.store(max_card_id, Ordering::Relaxed);
        self.next_deck_id.store(max_deck_id, Ordering::Relaxed);
        self.next_package_id
            .store(max_package_id, Ordering::Relaxed);
        Ok(())
    }

    pub fn next_deck_id(&self) -> u64 {
        self.next_deck_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn next_card_id(&self) -> u64 {
        self.next_card_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn next_package_id(&self) -> u64 {
        self.next_package_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn open_user_connection(&self) -> Result<Connection, String> {
        let db_path = self
            .user_db_path
            .get()
            .ok_or_else(|| "User database is not initialized yet".to_string())?;
        Connection::open(db_path).map_err(|e| format!("Failed to open user database: {e}"))
    }

    pub fn save_collection_card(&self, card: &Card) -> Result<(), String> {
        let connection = self.open_user_connection()?;
        let card_json = serde_json::to_string(card)
            .map_err(|e| format!("Failed to serialize collection card: {e}"))?;
        connection
            .execute(
                "INSERT INTO collection_cards (card_id, card_json)
                 VALUES (?1, ?2)
                 ON CONFLICT(card_id) DO UPDATE SET card_json = excluded.card_json",
                params![card.id(), card_json],
            )
            .map_err(|e| format!("Failed to persist collection card: {e}"))?;
        Ok(())
    }

    pub fn delete_collection_card(&self, card_id: u64) -> Result<(), String> {
        let connection = self.open_user_connection()?;
        connection
            .execute(
                "DELETE FROM collection_cards WHERE card_id = ?1",
                params![card_id],
            )
            .map_err(|e| format!("Failed to delete collection card: {e}"))?;
        connection
            .execute("DELETE FROM favorites WHERE card_id = ?1", params![card_id])
            .map_err(|e| format!("Failed to delete collection favorite: {e}"))?;
        Ok(())
    }

    pub fn save_favorite(&self, card_id: u64) -> Result<(), String> {
        let connection = self.open_user_connection()?;
        connection
            .execute(
                "INSERT OR IGNORE INTO favorites (card_id) VALUES (?1)",
                params![card_id],
            )
            .map_err(|e| format!("Failed to persist favorite: {e}"))?;
        Ok(())
    }

    pub fn delete_favorite(&self, card_id: u64) -> Result<(), String> {
        let connection = self.open_user_connection()?;
        connection
            .execute("DELETE FROM favorites WHERE card_id = ?1", params![card_id])
            .map_err(|e| format!("Failed to delete favorite: {e}"))?;
        Ok(())
    }

    pub fn save_deck(&self, deck: &Deck) -> Result<(), String> {
        let connection = self.open_user_connection()?;
        let deck_json =
            serde_json::to_string(deck).map_err(|e| format!("Failed to serialize deck: {e}"))?;
        connection
            .execute(
                "INSERT INTO decks (deck_id, name, deck_json)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(deck_id) DO UPDATE
                 SET name = excluded.name, deck_json = excluded.deck_json",
                params![deck.id(), deck.get_name(), deck_json],
            )
            .map_err(|e| format!("Failed to persist deck: {e}"))?;
        Ok(())
    }

    pub fn delete_deck(&self, deck_id: u64) -> Result<(), String> {
        let connection = self.open_user_connection()?;
        connection
            .execute("DELETE FROM decks WHERE deck_id = ?1", params![deck_id])
            .map_err(|e| format!("Failed to delete deck: {e}"))?;
        Ok(())
    }

    pub fn save_package(&self, package: &Package) -> Result<(), String> {
        let connection = self.open_user_connection()?;
        let package_json = serde_json::to_string(package)
            .map_err(|e| format!("Failed to serialize package: {e}"))?;
        connection
            .execute(
                "INSERT INTO packages (package_id, name, package_json)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(package_id) DO UPDATE
                 SET name = excluded.name, package_json = excluded.package_json",
                params![package.id(), package.get_name(), package_json],
            )
            .map_err(|e| format!("Failed to persist package: {e}"))?;
        Ok(())
    }

    pub fn delete_package(&self, package_id: u64) -> Result<(), String> {
        let connection = self.open_user_connection()?;
        connection
            .execute(
                "DELETE FROM packages WHERE package_id = ?1",
                params![package_id],
            )
            .map_err(|e| format!("Failed to delete package: {e}"))?;
        Ok(())
    }
}
