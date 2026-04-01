use crate::models::card_model::Card;
use crate::models::crispi_model::{self, Role, CrispiEvaluation};
use crate::state::AppState;
use tauri::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardRoles {
    pub card_id: u64,
    pub card_name: String,
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckRoleEvaluation {
    pub deck_id: u64,
    pub card_evaluations: Vec<CardRoles>,
    pub role_counts: HashMap<Role, usize>,
    pub crispi: CrispiEvaluation,
}

#[tauri::command]
pub fn evaluate_deck_roles(state: State<'_, AppState>, deck_id: u64) -> Result<DeckRoleEvaluation, String> {
    let decks = state.decks.read().map_err(|_| "Failed to acquire deck lock".to_string())?;
    let deck = decks.iter().find(|d| d.id() == deck_id).ok_or_else(|| format!("Deck {} not found", deck_id))?;

    let mut card_evaluations = Vec::new();
    let mut role_counts = HashMap::new();
    let mut total_mv = 0.0;
    let mut card_count = 0;

    let process_card = |card: &Card, card_evaluations: &mut Vec<CardRoles>, role_counts: &mut HashMap<Role, usize>, total_mv: &mut f32, card_count: &mut usize| {
        let roles_set = crispi_model::infer_roles(card);
        let mut roles: Vec<Role> = roles_set.into_iter().collect();
        roles.sort_by_key(|r| format!("{:?}", r));
        
        for role in &roles {
            *role_counts.entry(role.clone()).or_insert(0) += 1;
        }
        
        card_evaluations.push(CardRoles {
            card_id: card.id(),
            card_name: card.get_name().to_string(),
            roles,
        });

        if !card.is_land() {
            *total_mv += card.mana_value() as f32;
            *card_count += 1;
        }
    };

    // Process commander(s)
    match deck.get_commander() {
        crate::models::deck_model::CommanderSelection::None => {},
        crate::models::deck_model::CommanderSelection::Single(commander) => {
            process_card(commander, &mut card_evaluations, &mut role_counts, &mut total_mv, &mut card_count);
        },
        crate::models::deck_model::CommanderSelection::Partner(c1, c2) => {
            process_card(c1, &mut card_evaluations, &mut role_counts, &mut total_mv, &mut card_count);
            process_card(c2, &mut card_evaluations, &mut role_counts, &mut total_mv, &mut card_count);
        }
    }

    // Process mainboard
    for card in deck.get_cards() {
        process_card(card, &mut card_evaluations, &mut role_counts, &mut total_mv, &mut card_count);
    }

    let amv = if card_count > 0 { total_mv / card_count as f32 } else { 0.0 };
    let crispi = crispi_model::calculate_crispi(&role_counts, amv);

    Ok(DeckRoleEvaluation {
        deck_id,
        card_evaluations,
        role_counts,
        crispi,
    })
}
