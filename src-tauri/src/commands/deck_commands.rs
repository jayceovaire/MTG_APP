use crate::state::AppState;
use crate::models::deck_model::Deck;
use crate::commands::collection_commands::card_from_db_by_name;
use tauri::State;

// DECK COMMANDS
#[tauri::command]
pub fn create_deck(state: State<'_, AppState>, name: Option<String>) -> Result<Deck, String> {
    let mut deck = Deck::new(state.next_deck_id());
    let deck_name = match name {
        Some(name) if !name.trim().is_empty() => name,
        _ => "Untitled".to_string(),
    };
    deck.set_name(deck_name);
    state.decks.write().unwrap().push(deck.clone());
    state.save_deck(&deck)?;
    Ok(deck) //RETURNING THE DECK SO NEW DECK TILE CAN PULL FROM IT RATHER THAN RERENDER ALL THE DECKS IN UI

}
#[tauri::command]
pub fn rename_deck(state: State<'_, AppState>, deck_id: u64, name: Option<String>) -> Result<Deck, String> {
    let deck_name = match name {
        Some(name) if !name.trim().is_empty() => name,
        _ => "Untitled".to_string(),
    };

    let mut decks = state.decks.write().map_err(|_| "Failed to acquire deck lock".to_string())?;
    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.set_name(deck_name);
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn get_decks(state: State<'_, AppState>) -> Vec<Deck> {
    state.decks.read().unwrap().clone()
}

#[tauri::command]
pub fn get_deck(state: State<'_, AppState>, deck_id: u64) -> Result<Deck, String> {
    state
        .decks
        .read()
        .map_err(|_| "Failed to acquire deck lock".to_string())?
        .iter()
        .find(|deck| deck.id() == deck_id)
        .cloned()
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))
}


// TEST COMMAND
#[tauri::command]
pub fn test_command(greeting: Option<String>) -> String {
    greeting.unwrap_or("greeting".to_string())
}

#[tauri::command]
pub fn delete_deck(state: State<'_, AppState>, deck_id: u64) -> Result<(), String> {

    let mut decks = state.decks.write().map_err(|_| "Failed to acquire deck lock".to_string())?;

    let index = decks
        .iter()
        .position(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    decks.remove(index);
    drop(decks);
    state.delete_deck(deck_id)?;
    Ok(())
}

#[tauri::command]
pub fn duplicate_deck(state: State<'_, AppState>, deck_id: u64) -> Result<Deck, String> {
    let mut decks = state.decks.write().map_err(|_| "Failed to acquire deck lock".to_string())?;

    let source_deck = decks
        .iter()
        .find(|deck| deck.id() == deck_id)
        .cloned()
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    let mut duplicated_deck = source_deck;
    duplicated_deck.set_id(state.next_deck_id());
    let base_name = duplicated_deck.get_name().to_string();
    duplicated_deck.set_name(format!("{} (Copy)", base_name));

    decks.push(duplicated_deck.clone());
    drop(decks);
    state.save_deck(&duplicated_deck)?;
    Ok(duplicated_deck)
}

#[tauri::command]
pub fn add_card_to_deck(state: State<'_, AppState>, deck_id: u64, name: String) -> Result<Deck, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Card name cannot be empty".to_string());
    }

    let new_card = card_from_db_by_name(trimmed, state.next_card_id())?
        .ok_or_else(|| format!("Card '{trimmed}' not found in local database"))?;

    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.add_card(new_card);
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn remove_card_from_deck(state: State<'_, AppState>, deck_id: u64, card_id: u64) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    let card_index = deck
        .get_cards()
        .iter()
        .position(|card| card.id() == card_id)
        .ok_or_else(|| format!("Card with id {} not found in deck {}", card_id, deck_id))?;

    if !deck.remove_card_at(card_index) {
        return Err(format!("Failed to remove card with id {} from deck {}", card_id, deck_id));
    }

    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn set_deck_commander(state: State<'_, AppState>, deck_id: u64, card_id: u64) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.set_single_commander_from_deck(card_id)?;
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn set_deck_partner(state: State<'_, AppState>, deck_id: u64, card_id: u64) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.set_partner_commander_from_deck(card_id)?;
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn remove_deck_partner(state: State<'_, AppState>,deck_id: u64, card_id: u64) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
    .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.remove_partner_commander_from_deck(card_id)?;
    let updated = deck.clone();
    drop(decks);
    state.save_deck(&updated)?;
    Ok(updated)
}



#[tauri::command]
pub fn remove_deck_commander(state: State<'_, AppState>, deck_id: u64) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.clear_commander_to_deck()?;
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_deck_commander(state: State<'_, AppState>, deck_id: u64) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.remove_commander()?;
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}
