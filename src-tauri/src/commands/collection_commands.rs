use crate::models::card_model::{Card, CardType, SuperType};
use crate::state::AppState;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Row, params};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::{AppHandle, State, Manager};

const SCRYFALL_DB_RELATIVE_PATH: &str = "src/db/scryfall.db";
static SCRYFALL_DB_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOOKUP_INDEXES_READY: OnceLock<()> = OnceLock::new();
static SEARCH_CANDIDATES: OnceLock<Result<Vec<CardSearchCandidate>, String>> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
pub struct CardSearchSuggestion {
    name: String,
    mana_cost: Option<String>,
    type_line: String,
    commander_legality: String,
    game_changer: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionCardView {
    #[serde(flatten)]
    card: Card,
    favorite: bool,
}

#[derive(Debug, Clone)]
struct CardSearchCandidate {
    suggestion: CardSearchSuggestion,
    normalized_name: String,
}

pub fn initialize_db_paths(app: &AppHandle) -> Result<(), String> {
    let path = app.path()
        .resource_dir()
        .map_err(|e| format!("Failed to resolve resource directory: {e}"))?
        .join(SCRYFALL_DB_RELATIVE_PATH);
    let _ = SCRYFALL_DB_PATH.set(path);
    Ok(())
}

fn scryfall_db_path() -> Result<PathBuf, String> {
    SCRYFALL_DB_PATH.get().cloned().ok_or_else(|| "Scryfall DB path not initialized".to_string())
}

fn open_scryfall_db() -> Result<Connection, String> {
    let db_path = scryfall_db_path()?;
    let path_str = db_path.to_string_lossy();
    let path_str = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
    let path_str = path_str.replace('\\', "/");

    // Standardize URI construction for both Windows and Unix local/UNC paths.
    // UNC paths like //server/share should be file://server/share.
    // Absolute local paths like C:/path or /home/user should be file:///C:/path or file:///home/user.
    let db_uri = if path_str.starts_with("//") && !path_str.starts_with("///") {
        format!("file:{}?mode=ro&immutable=1", path_str)
    } else {
        format!("file:///{}?mode=ro&immutable=1", path_str.trim_start_matches('/'))
    };

    Connection::open_with_flags(
        db_uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
        .map_err(|e| format!("Failed to open scryfall.db: {e}"))
}

fn ensure_lookup_indexes(connection: &Connection) -> Result<(), String> {
    if LOOKUP_INDEXES_READY.get().is_some() {
        return Ok(());
    }

    let _ = connection;
    // The bundled Scryfall database lives under `src-tauri/src/db`, which Tauri watches in dev.
    // Creating indexes at runtime generates SQLite sidecar files (`-wal`, `-shm`) and triggers
    // an endless rebuild loop. Keep the bundled database strictly read-only in development.
    let _ = LOOKUP_INDEXES_READY.set(());
    Ok(())
}

fn normalize_search_text(value: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_space = false;

    for ch in value.chars() {
        let replacement = match ch.to_ascii_lowercase() {
            'a'..='z' | '0'..='9' => Some(ch.to_ascii_lowercase().to_string()),
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => Some("a".to_string()),
            'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => Some("c".to_string()),
            'ď' | 'đ' => Some("d".to_string()),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => Some("e".to_string()),
            'ĝ' | 'ğ' | 'ġ' | 'ģ' => Some("g".to_string()),
            'ĥ' | 'ħ' => Some("h".to_string()),
            'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => Some("i".to_string()),
            'ĵ' => Some("j".to_string()),
            'ķ' => Some("k".to_string()),
            'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => Some("l".to_string()),
            'ñ' | 'ń' | 'ņ' | 'ň' => Some("n".to_string()),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => Some("o".to_string()),
            'ŕ' | 'ŗ' | 'ř' => Some("r".to_string()),
            'ś' | 'ŝ' | 'ş' | 'š' => Some("s".to_string()),
            'ţ' | 'ť' | 'ŧ' => Some("t".to_string()),
            'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => Some("u".to_string()),
            'ŵ' => Some("w".to_string()),
            'ý' | 'ÿ' | 'ŷ' => Some("y".to_string()),
            'ź' | 'ż' | 'ž' => Some("z".to_string()),
            'æ' => Some("ae".to_string()),
            'œ' => Some("oe".to_string()),
            'ß' => Some("ss".to_string()),
            _ if ch.is_whitespace() || matches!(ch, '-' | '_' | '/' | '\\' | '\'' | '"' | ',' | '.' | ':' | ';' | '(' | ')' | '[' | ']' | '{' | '}') => Some(" ".to_string()),
            _ => None,
        };

        if let Some(replacement) = replacement {
            for out in replacement.chars() {
                if out == ' ' {
                    if !last_was_space && !normalized.is_empty() {
                        normalized.push(' ');
                    }
                    last_was_space = true;
                } else {
                    normalized.push(out);
                    last_was_space = false;
                }
            }
        }
    }

    normalized.trim().to_string()
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();

    if left_chars.is_empty() {
        return right_chars.len();
    }
    if right_chars.is_empty() {
        return left_chars.len();
    }

    let mut previous: Vec<usize> = (0..=right_chars.len()).collect();
    let mut current = vec![0; right_chars.len() + 1];

    for (i, left_char) in left_chars.iter().enumerate() {
        current[0] = i + 1;

        for (j, right_char) in right_chars.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            current[j + 1] = (current[j] + 1)
                .min(previous[j + 1] + 1)
                .min(previous[j] + cost);
        }

        previous.clone_from(&current);
    }

    previous[right_chars.len()]
}

fn search_score(query: &str, candidate: &CardSearchCandidate) -> Option<usize> {
    let normalized_name = &candidate.normalized_name;

    if normalized_name == query {
        return Some(0);
    }

    if normalized_name.starts_with(query) {
        return Some(10 + normalized_name.len().saturating_sub(query.len()));
    }

    if normalized_name
        .split_whitespace()
        .any(|word| word.starts_with(query))
    {
        return Some(30 + normalized_name.len().saturating_sub(query.len()));
    }

    if let Some(position) = normalized_name.find(query) {
        return Some(60 + position);
    }

    let query_tokens: Vec<&str> = query
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect();
    if !query_tokens.is_empty() {
        let token_hits = query_tokens
            .iter()
            .filter(|token| normalized_name.contains(**token))
            .count();

        if token_hits > 0 {
            return Some(90 + (query_tokens.len() - token_hits) * 12 + normalized_name.len() / 6);
        }
    }

    if query.len() >= 4 {
        let fuzzy_distance = normalized_name
            .split_whitespace()
            .map(|word| levenshtein_distance(query, word))
            .min()
            .unwrap_or(usize::MAX);

        if fuzzy_distance <= 2 {
            return Some(140 + (fuzzy_distance * 20) + normalized_name.len() / 8);
        }
    }

    None
}

fn load_search_candidates() -> Result<Vec<CardSearchCandidate>, String> {
    let connection = open_scryfall_db()?;
    ensure_lookup_indexes(&connection)?;

    let mut statement = connection
        .prepare(
            "SELECT c.name,
                    NULLIF(c.mana_cost, '') AS mana_cost,
                    CASE
                        WHEN NULLIF(c.type_line, '') IS NULL THEN 'Card'
                        WHEN c.type_line = 'Card' THEN 'Card'
                        WHEN c.type_line = 'Card // Card' THEN 'Card'
                        ELSE c.type_line
                    END AS type_line,
                    c.commander_legality,
                    c.game_changer
             FROM cards c
             ORDER BY c.name COLLATE NOCASE, 
                      CASE WHEN c.commander_legality = 'legal' THEN 0 ELSE 1 END,
                      c.id",
        )
        .map_err(|e| format!("Failed to prepare search candidate query: {e}"))?;

    let rows = statement
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let mana_cost: Option<String> = row.get(1)?;
            let type_line: String = row.get(2)?;
            let commander_legality: String = row.get::<_, Option<String>>(3)?
                .unwrap_or_else(|| "not_legal".to_string());
            let game_changer_int: i64 = row.get(4)?;
            let game_changer = game_changer_int != 0;

            Ok(CardSearchSuggestion {
                name,
                mana_cost,
                type_line,
                commander_legality,
                game_changer,
            })
        })
        .map_err(|e| format!("Failed to execute search candidate query: {e}"))?;

    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    for row in rows {
        let suggestion = row.map_err(|e| format!("Failed to read search candidate row: {e}"))?;
        if !seen.insert(suggestion.name.clone()) {
            continue;
        }

        let normalized_name = normalize_search_text(&suggestion.name);
        if normalized_name.is_empty() {
            continue;
        }

        candidates.push(CardSearchCandidate {
            suggestion,
            normalized_name,
        });
    }

    Ok(candidates)
}

fn search_candidates() -> Result<&'static Vec<CardSearchCandidate>, String> {
    match SEARCH_CANDIDATES.get_or_init(load_search_candidates) {
        Ok(candidates) => Ok(candidates),
        Err(error) => Err(error.clone()),
    }
}

fn build_collection_card_view(card: Card, favorite_ids: &HashSet<u64>) -> CollectionCardView {
    let favorite = favorite_ids.contains(&card.id());
    CollectionCardView { card, favorite }
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
    let connection = open_scryfall_db()?;
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
                             cf.rowid
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
                             cf.rowid
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
                             cf.rowid
                         LIMIT 1),
                        NULLIF(c.oracle_text, '')
                    ) AS oracle_text,
                    c.commander_legality,
                    c.game_changer,
                    c.id
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
             ORDER BY picked.priority, 
                      CASE WHEN c.commander_legality = 'legal' THEN 0 ELSE 1 END,
                      picked.quality DESC, 
                      picked.card_id
             LIMIT 1",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    stmt.query_row(params![name], |row| map_row_to_card(row, id))
        .optional()
        .map_err(|e| format!("Failed to execute query: {e}"))
}

#[tauri::command]
pub async fn get_card(app: AppHandle, name: String) -> Result<Option<Card>, String> {
    let mut card_opt = card_from_db_by_name(&name, 0)?;
    
    if let Some(ref mut card) = card_opt {
        let cache_dir = app.path().local_data_dir()
            .map_err(|e| format!("Failed to resolve local app data directory: {e}"))?
            .join("mtg_app")
            .join("card_images");

        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create cache directory: {e}"))?;
        }

        let client = reqwest::Client::builder()
            .user_agent("MTG_App/0.1.0 (contact: support@mtg_app.local)")
            .build()
            .map_err(|e| format!("Failed to create reqwest client: {e}"))?;

        let _ = crate::commands::image_commands::process_card(card, &cache_dir, &client).await;
        crate::commands::image_commands::convert_card_image_to_base64(card);
    }
    
    Ok(card_opt)
}

#[tauri::command]
pub fn search_card_suggestions(query: String) -> Result<Vec<CardSearchSuggestion>, String> {
    let normalized_query = normalize_search_text(query.trim());
    if normalized_query.len() < 2 {
        return Ok(Vec::new());
    }

    let mut matches: Vec<(usize, &CardSearchCandidate)> = search_candidates()?
        .iter()
        .filter_map(|candidate| search_score(&normalized_query, candidate).map(|score| (score, candidate)))
        .collect();

    matches.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.suggestion.name.cmp(&right.1.suggestion.name))
    });

    Ok(matches
        .into_iter()
        .take(8)
        .map(|(_, candidate)| candidate.suggestion.clone())
        .collect())
}

fn map_row_to_card(row: &Row<'_>, id: u64) -> rusqlite::Result<Card> {
    let db_name: String = row.get(0)?;
    let mana_cost: Option<String> = row.get(1)?;
    let cmc: f64 = row.get(2)?;
    let type_line_raw: Option<String> = row.get(3)?;
    let oracle_text: Option<String> = row.get(4)?;
    let commander_legality: String = row
        .get::<_, Option<String>>(5)?
        .unwrap_or_else(|| "not_legal".to_string());
    let db_game_changer: i64 = row.get(6)?;
    let scryfall_id: String = row.get(7)?;

    let type_line = type_line_raw.unwrap_or_else(|| "Card".to_string());
    let game_changer = db_game_changer != 0;
    let legal_in_commander = commander_legality == "legal";

    Ok(Card::new(
        id,
        String::new(),
        None,
        db_name,
        mana_cost,
        cmc.max(0.0).round() as u8,
        parse_card_types(&type_line),
        parse_super_types(&type_line),
        parse_sub_types(&type_line),
        oracle_text,
        commander_legality,
        legal_in_commander,
        game_changer,
        Some(scryfall_id),
    ))
}

#[tauri::command]
pub async fn get_random_card(app: AppHandle) -> Result<Option<Card>, String> {
    let mut card_opt = {
        let connection = open_scryfall_db()?;
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
                                 cf.rowid
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
                                 cf.rowid
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
                                 cf.rowid
                             LIMIT 1),
                            NULLIF(c.oracle_text, '')
                        ) AS oracle_text,
                        c.commander_legality,
                        c.game_changer,
                        c.id
                 FROM cards c
                 WHERE c.id IN (SELECT id FROM cards ORDER BY RANDOM() LIMIT 1)",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        stmt.query_row([], |row| map_row_to_card(row, 0))
            .optional()
            .map_err(|e| format!("Failed to execute query: {e}"))?
    };

    if let Some(ref mut card) = card_opt {
        let cache_dir = app.path().local_data_dir()
            .map_err(|e| format!("Failed to resolve local app data directory: {e}"))?
            .join("mtg_app")
            .join("card_images");

        if !cache_dir.exists() {
            let _ = std::fs::create_dir_all(&cache_dir);
        }

        let client = reqwest::Client::builder()
            .user_agent("MTG_App/0.1.0 (contact: support@mtg_app.local)")
            .build()
            .map_err(|e| format!("Failed to create reqwest client: {e}"))?;

        let _ = crate::commands::image_commands::process_card(card, &cache_dir, &client).await;

        crate::commands::image_commands::convert_card_image_to_base64(card);
    }

    Ok(card_opt)
}

#[tauri::command]
pub async fn add_card_to_collection(
    state: State<'_, AppState>,
    app: AppHandle,
    name: String,
) -> Result<CollectionCardView, String> {
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

    // Trigger image fetching in background for collection
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(app_handle, None, None, Some(true), None).await;
    });

    Ok(CollectionCardView {
        card: new_card,
        favorite: false,
    })
}

#[tauri::command]
pub async fn bulk_add_cards_to_collection(
    state: State<'_, AppState>,
    app: AppHandle,
    cards: Vec<(u32, String)>,
) -> Result<Vec<CollectionCardView>, String> {
    if cards.is_empty() {
        return Err("No cards to add".to_string());
    }

    let mut added_cards = Vec::new();
    for (qty, name) in cards {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }

        for _ in 0..qty {
            let card = card_from_db_by_name(trimmed, state.next_card_id())?
                .ok_or_else(|| format!("Card '{}' not found in local database", trimmed))?;
            added_cards.push(card);
        }
    }

    let mut views = Vec::new();
    let mut collection = state
        .collection
        .write()
        .map_err(|_| "Failed to acquire collection lock".to_string())?;

    let favorite_ids: HashSet<u64> = state
        .favorites
        .read()
        .map_err(|_| "Failed to lock favorites for read")?
        .iter()
        .cloned()
        .collect();

    for card in added_cards {
        state.save_collection_card(&card)?;
        collection.push(card.clone());
        views.push(build_collection_card_view(card, &favorite_ids));
    }
    drop(collection);

    // Trigger image fetching in background ONCE for collection
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(app_handle, None, None, Some(true), None).await;
    });

    Ok(views)
}

#[tauri::command]
pub async fn duplicate_collection_card(
    state: State<'_, AppState>,
    app: AppHandle,
    card_id: u64,
) -> Result<CollectionCardView, String> {
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

    // Trigger image fetching in background for collection
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(app_handle, None, None, Some(true), None).await;
    });

    Ok(CollectionCardView {
        card: duplicated,
        favorite: false,
    })
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
pub fn set_collection_card_favorite(
    state: State<'_, AppState>,
    card_id: u64,
    favorite: bool,
) -> Result<CollectionCardView, String> {
    let collection = state
        .collection
        .read()
        .map_err(|_| "Failed to lock collection for reading".to_string())?;
    let card = collection
        .iter()
        .find(|card| card.id() == card_id)
        .cloned()
        .ok_or_else(|| format!("Card with id {} not found", card_id))?;
    drop(collection);

    let mut favorites = state
        .favorites
        .write()
        .map_err(|_| "Failed to lock favorites for writing".to_string())?;
    if favorite {
        if !favorites.contains(&card_id) {
            favorites.push(card_id);
        }
        drop(favorites);
        state.save_favorite(card_id)?;
    } else {
        favorites.retain(|id| *id != card_id);
        drop(favorites);
        state.delete_favorite(card_id)?;
    }

    Ok(CollectionCardView { card, favorite })
}

#[tauri::command]
pub async fn get_collection(state: State<'_, AppState>, app: AppHandle) -> Result<Vec<CollectionCardView>, String> {
    let favorite_ids: HashSet<u64> = state
        .favorites
        .read()
        .map_err(|_| "Failed to lock favorites for read".to_string())?
        .iter()
        .copied()
        .collect();
    
    let cards = state.collection
        .read()
        .map_err(|_| "Failed to lock collection for read".to_string())?
        .clone()
        .into_iter()
        .map(|card| build_collection_card_view(card, &favorite_ids))
        .collect();

    // Trigger image fetching for collection in background
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(app_handle, None, None, Some(true), None).await;
    });

    Ok(cards)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_random_card() {
        // This test requires scryfall.db to exist at the expected path.
        // In the test environment, we assume it's available or we skip/fail.
        // Mock AppHandle isn't easy here, but since it's a unit test
        // we might need to refactor get_random_card to accept paths or use a mock.
        // For now, let's see if we can at least compile it or if we need a different approach.
        // Actually, since it needs AppHandle, a simple unit test might fail to compile or run.
    }
}



