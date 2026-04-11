use crate::models::card_model::CardType;
use crate::models::crispi_model::{self, CrispiEvaluation, Role};
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
pub fn evaluate_deck_roles(
    state: State<'_, AppState>,
    deck_id: u64,
) -> Result<DeckRoleEvaluation, String> {
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

    let combo_piece_names = crispi_model::combo_piece_names_for_deck(&mainboard, &commanders);

    let mut all_cards = Vec::new();
    all_cards.extend(commanders.clone());
    all_cards.extend(mainboard.clone());

    let mut card_evaluations = Vec::new();
    let mut role_counts = HashMap::new();

    let integration_results = crate::models::crispi_integration::compute_integration(&all_cards);

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

    let n_gc = deck.get_game_changer_count();
    let crispi = crispi_model::calculate_crispi(&mainboard, &commanders, n_gc);
    drop(decks);

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
