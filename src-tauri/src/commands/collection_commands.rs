use crate::models::card_model::{Card, CardType, SuperType};
use crate::state::AppState;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::State;

const SCRYFALL_DB_RELATIVE_PATH: &str = "src/db/scryfall.db";
static LOOKUP_INDEXES_READY: OnceLock<()> = OnceLock::new();

fn scryfall_db_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SCRYFALL_DB_RELATIVE_PATH)
}

fn ensure_lookup_indexes(connection: &Connection) -> Result<(), String> {
    if LOOKUP_INDEXES_READY.get().is_some() {
        return Ok(());
    }

    connection
        .execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_cards_name_nocase
             ON cards(name COLLATE NOCASE);

             CREATE INDEX IF NOT EXISTS idx_card_faces_name_nocase_card_id
             ON card_faces(name COLLATE NOCASE, card_id);",
        )
        .map_err(|e| format!("Failed to create lookup indexes: {e}"))?;

    let _ = LOOKUP_INDEXES_READY.set(());
    Ok(())
}

fn parse_card_types(type_line: &str) -> Vec<CardType> {
    let mut types = Vec::new();
    if type_line.contains("Creature") {
        types.push(CardType::Creature);
    }
    if type_line.contains("Instant") {
        types.push(CardType::Instant);
    }
    if type_line.contains("Sorcery") {
        types.push(CardType::Sorcery);
    }
    if type_line.contains("Enchantment") {
        types.push(CardType::Enchantment);
    }
    if type_line.contains("Artifact") {
        types.push(CardType::Artifact);
    }
    if type_line.contains("Land") {
        types.push(CardType::Land);
    }
    if type_line.contains("Planeswalker") {
        types.push(CardType::Planeswalker);
    }
    if type_line.contains("Battle") {
        types.push(CardType::Battle);
    }
    if type_line.contains("Tribal") {
        types.push(CardType::Tribal);
    }
    types
}

fn parse_super_types(type_line: &str) -> Vec<SuperType> {
    let mut supertypes = Vec::new();
    if type_line.contains("Legendary") {
        supertypes.push(SuperType::Legendary);
    }
    if type_line.contains("Basic") {
        supertypes.push(SuperType::Basic);
    }
    if type_line.contains("Snow") {
        supertypes.push(SuperType::Snow);
    }
    if type_line.contains("World") {
        supertypes.push(SuperType::World);
    }
    if type_line.contains("Ongoing") {
        supertypes.push(SuperType::Ongoing);
    }
    supertypes
}

fn parse_sub_types(type_line: &str) -> Vec<String> {
    let right_side = type_line
        .split_once("—")
        .map(|(_, subtype_text)| subtype_text)
        .or_else(|| type_line.split_once(" - ").map(|(_, subtype_text)| subtype_text));

    match right_side {
        Some(subtype_text) => subtype_text
            .split_whitespace()
            .map(str::trim)
            .filter(|subtype| !subtype.is_empty())
            .map(ToString::to_string)
            .collect(),
        None => Vec::new(),
    }
}

pub(crate) fn card_from_db_by_name(name: &str, id: u64) -> Result<Option<Card>, String> {
    let connection = Connection::open(scryfall_db_path())
        .map_err(|e| format!("Failed to open scryfall.db: {e}"))?;
    ensure_lookup_indexes(&connection)?;

    let mut stmt = connection
        .prepare(
            "SELECT c.name AS name,
                    COALESCE(
                        (SELECT NULLIF(cf.mana_cost, '')
                         FROM card_faces cf
                         WHERE cf.card_id = c.id
                         ORDER BY
                             CASE WHEN NULLIF(cf.oracle_text, '') IS NOT NULL THEN 0 ELSE 1 END,
                             CASE
                                 WHEN NULLIF(cf.type_line, '') IS NOT NULL
                                  AND cf.type_line <> 'Card'
                                  AND cf.type_line <> 'Card // Card' THEN 0
                                 ELSE 1
                             END,
                             CASE WHEN NULLIF(cf.mana_cost, '') IS NOT NULL THEN 0 ELSE 1 END,
                             cf.id
                         LIMIT 1),
                        NULLIF(c.mana_cost, '')
                    ) AS mana_cost,
                    c.cmc,
                    COALESCE(
                        (SELECT CASE
                                    WHEN NULLIF(cf.type_line, '') IS NULL THEN NULL
                                    WHEN cf.type_line = 'Card' THEN NULL
                                    WHEN cf.type_line = 'Card // Card' THEN NULL
                                    ELSE cf.type_line
                                END
                         FROM card_faces cf
                         WHERE cf.card_id = c.id
                         ORDER BY
                             CASE WHEN NULLIF(cf.oracle_text, '') IS NOT NULL THEN 0 ELSE 1 END,
                             CASE
                                 WHEN NULLIF(cf.type_line, '') IS NOT NULL
                                  AND cf.type_line <> 'Card'
                                  AND cf.type_line <> 'Card // Card' THEN 0
                                 ELSE 1
                             END,
                             CASE WHEN NULLIF(cf.mana_cost, '') IS NOT NULL THEN 0 ELSE 1 END,
                             cf.id
                         LIMIT 1),
                        CASE
                            WHEN NULLIF(c.type_line, '') IS NULL THEN NULL
                            WHEN c.type_line = 'Card' THEN NULL
                            WHEN c.type_line = 'Card // Card' THEN NULL
                            ELSE c.type_line
                        END
                    ) AS type_line,
                    COALESCE(
                        (SELECT NULLIF(cf.oracle_text, '')
                         FROM card_faces cf
                         WHERE cf.card_id = c.id
                         ORDER BY
                             CASE WHEN NULLIF(cf.oracle_text, '') IS NOT NULL THEN 0 ELSE 1 END,
                             CASE
                                 WHEN NULLIF(cf.type_line, '') IS NOT NULL
                                  AND cf.type_line <> 'Card'
                                  AND cf.type_line <> 'Card // Card' THEN 0
                                 ELSE 1
                             END,
                             CASE WHEN NULLIF(cf.mana_cost, '') IS NOT NULL THEN 0 ELSE 1 END,
                             cf.id
                         LIMIT 1),
                        NULLIF(c.oracle_text, '')
                    ) AS oracle_text
             FROM (
                 SELECT
                     c.id AS card_id,
                     0 AS priority,
                     (CASE WHEN NULLIF(c.oracle_text, '') IS NOT NULL THEN 100 ELSE 0 END) +
                     (CASE
                         WHEN NULLIF(c.type_line, '') IS NOT NULL
                          AND c.type_line <> 'Card'
                          AND c.type_line <> 'Card // Card' THEN 10
                         ELSE 0
                      END) +
                     (CASE WHEN NULLIF(c.mana_cost, '') IS NOT NULL THEN 1 ELSE 0 END) AS quality
                 FROM cards c
                 WHERE c.name = ?1 COLLATE NOCASE

                 UNION ALL

                 SELECT
                     c.id AS card_id,
                     1 AS priority,
                     MAX(
                         (CASE WHEN NULLIF(cf.oracle_text, '') IS NOT NULL THEN 100 ELSE 0 END) +
                         (CASE
                             WHEN NULLIF(cf.type_line, '') IS NOT NULL
                              AND cf.type_line <> 'Card'
                              AND cf.type_line <> 'Card // Card' THEN 10
                             ELSE 0
                          END) +
                         (CASE WHEN NULLIF(cf.mana_cost, '') IS NOT NULL THEN 1 ELSE 0 END)
                     ) AS quality
                 FROM card_faces cf
                 JOIN cards c ON c.id = cf.card_id
                 WHERE cf.name = ?1 COLLATE NOCASE
                 GROUP BY c.id
             ) picked
             JOIN cards c ON c.id = picked.card_id
             ORDER BY picked.priority, picked.quality DESC, picked.card_id
             LIMIT 1",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    stmt.query_row(params![name], |row| {
        let db_name: String = row.get(0)?;
        let mana_cost: Option<String> = row.get(1)?;
        let cmc: f64 = row.get(2)?;
        let type_line: Option<String> = row.get(3)?;
        let oracle_text: Option<String> = row.get(4)?;
        let type_line = type_line.unwrap_or_else(|| "Card".to_string());

        Ok(Card::new(
            id,
            String::new(),
            db_name,
            mana_cost,
            cmc.max(0.0).round() as u8,
            parse_card_types(&type_line),
            parse_super_types(&type_line),
            parse_sub_types(&type_line),
            oracle_text,
        ))
    })
    .optional()
    .map_err(|e| format!("Failed to execute query: {e}"))
}

#[tauri::command]
pub fn add_card_to_collection(state: State<'_, AppState>, name: String) -> Result<Card, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Card name cannot be empty".to_string());
    }

    let new_card = card_from_db_by_name(trimmed, state.next_card_id())?
        .ok_or_else(|| format!("Card '{trimmed}' not found in local database"))?;

    let mut collection = state
        .collection
        .write()
        .map_err(|_| "Failed to lock collection for writing".to_string())?;
    collection.push(new_card.clone());
    drop(collection);
    state.save_collection_card(&new_card)?;

    Ok(new_card)
}

#[tauri::command]
pub fn duplicate_collection_card(state: State<'_, AppState>, card_id: u64) -> Result<Card, String> {
    let mut collection = state
        .collection
        .write()
        .map_err(|_| "Failed to lock collection for writing".to_string())?;

    let source_card = collection
        .iter()
        .find(|card| card.id() == card_id)
        .cloned()
        .ok_or_else(|| format!("Card with id {} not found", card_id))?;

    let mut duplicated = source_card;
    duplicated.set_id(state.next_card_id());
    collection.push(duplicated.clone());
    drop(collection);
    state.save_collection_card(&duplicated)?;

    Ok(duplicated)
}

#[tauri::command]
pub fn remove_collection_card(state: State<'_, AppState>, card_id: u64) -> Result<(), String> {
    let mut collection = state
        .collection
        .write()
        .map_err(|_| "Failed to lock collection for writing".to_string())?;

    let index = collection
        .iter()
        .position(|card| card.id() == card_id)
        .ok_or_else(|| format!("Card with id {} not found", card_id))?;

    collection.remove(index);
    drop(collection);
    state.delete_collection_card(card_id)?;
    Ok(())
}


#[tauri::command]
pub fn get_collection(state: State<'_, AppState>) -> Vec<Card> {
    state.collection.read().unwrap().clone()
}



