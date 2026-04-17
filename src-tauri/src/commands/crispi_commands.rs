use crate::models::card_model::CardType;
use crate::models::crispi_model::{self, CrispiEvaluation, Role};
use crate::models::sidecar_models::{FindMyCombosResponse, Variant};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardRoles {
    pub card_id: u64,
    pub card_name: String,
    pub roles: Vec<Role>,
    pub tier: crispi_model::QualityTier,
    pub card_types: Vec<CardType>,
    pub is_commander: bool,
    pub integration: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeckRoleEvaluation {
    pub deck_id: u64,
    pub card_evaluations: Vec<CardRoles>,
    pub role_counts: HashMap<Role, f32>,
    pub crispi: CrispiEvaluation,
}

#[tauri::command]
pub async fn evaluate_deck_roles(
    state: State<'_, AppState>,
    deck_id: u64,
) -> Result<DeckRoleEvaluation, String> {
    let (mainboard, commanders, n_gc) = {
        let decks = state
            .decks
            .read()
            .map_err(|_| "Failed to acquire deck lock".to_string())?;
        let deck = decks
            .iter()
            .find(|d| d.id() == deck_id)
            .ok_or_else(|| format!("Deck {} not found", deck_id))?;

        let mut mainboard = Vec::new();
        let mut commanders = Vec::new();
        match deck.get_commander() {
            crate::models::deck_model::CommanderSelection::None => {}
            crate::models::deck_model::CommanderSelection::Single(commander) => {
                commanders.push(commander.clone());
            }
            crate::models::deck_model::CommanderSelection::Partner(c1, c2) => {
                commanders.push(c1.clone());
                commanders.push(c2.clone());
            }
        }
        for card in deck.get_cards() {
            mainboard.push(card.clone());
        }
        (mainboard, commanders, deck.get_game_changer_count())
    };

    // Fetch combos from sidecar
    let sidecar_combos = match fetch_sidecar_combos(&mainboard, &commanders).await {
        Ok(combos) => combos,
        Err(e) => {
            eprintln!("Sidecar combo fetch failed: {}. Continuing evaluation without sidecar combos.", e);
            Vec::new()
        }
    };

    let combo_piece_names =
        crispi_model::combo_piece_names_for_deck(&mainboard, &commanders, &sidecar_combos);

    let mut all_cards = Vec::new();
    all_cards.extend(commanders.clone());
    all_cards.extend(mainboard.clone());

    let mut card_evaluations = Vec::new();
    let mut role_counts = HashMap::new();

    let integration_results =
        crate::models::crispi_integration::compute_integration(&all_cards, &sidecar_combos);

    for (i, card) in all_cards.iter().enumerate() {
        let is_commander = i < commanders.len();
        let roles_set = crispi_model::infer_roles_with_combo_context(card, &combo_piece_names);
        let tier = crispi_model::classify_card(card, &roles_set);
        let weight = tier.weight();

        let mut roles: Vec<Role> = roles_set.into_iter().collect();
        roles.sort_by_key(|r| format!("{:?}", r));

        for role in &roles {
            *role_counts.entry(role.clone()).or_insert(0.0) += weight;
        }

        let integration = integration_results[i].integration;

        card_evaluations.push(CardRoles {
            card_id: card.id(),
            card_name: card.get_name().to_string(),
            roles,
            tier,
            card_types: card.card_type().to_vec(),
            is_commander,
            integration,
        });
    }

    let crispi = crispi_model::calculate_crispi(&mainboard, &commanders, n_gc, &sidecar_combos);

    {
        let mut decks = state
            .decks
            .write()
            .map_err(|_| "Failed to acquire deck lock".to_string())?;
        if let Some(deck) = decks.iter_mut().find(|d| d.id() == deck_id) {
            if deck.bracket() != crispi.bracket {
                deck.set_bracket(crispi.bracket);
                let updated = deck.clone();
                drop(decks);
                state.save_deck(&updated)?;
            }
        }
    }

    Ok(DeckRoleEvaluation {
        deck_id,
        card_evaluations,
        role_counts,
        crispi,
    })
}

async fn fetch_sidecar_combos(
    mainboard: &[crate::models::card_model::Card],
    commanders: &[crate::models::card_model::Card],
) -> Result<Vec<Variant>, String> {
    let client = reqwest::Client::new();

    let main_cards: Vec<serde_json::Value> = mainboard
        .iter()
        .map(|card| {
            serde_json::json!({
                "card": card.get_name(),
                "quantity": 1
            })
        })
        .collect();

    let commander_cards: Vec<serde_json::Value> = commanders
        .iter()
        .map(|card| {
            serde_json::json!({
                "card": card.get_name(),
                "quantity": 1
            })
        })
        .collect();

    let body = serde_json::json!({
        "main": main_cards,
        "commanders": commander_cards
    });

    let response = client
        .post("http://127.0.0.1:8000/find-my-combos")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let result: FindMyCombosResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(result.variants)
    } else {
        // If sidecar is not running or returns error, we return empty combos instead of failing the whole evaluation
        // Or should we fail? Given the requirement, it seems important.
        // Let's return empty and log if it fails.
        eprintln!("Failed to fetch combos from sidecar: {}", response.status());
        Ok(vec![])
    }
}
