use crate::commands::collection_commands::card_from_db_by_name;
use crate::models::deck_model::Deck;
use crate::models::package_model::Package;
use crate::state::AppState;
use tauri::{AppHandle, State};

// DECK COMMANDS
#[tauri::command]
pub fn create_deck(state: State<'_, AppState>, name: Option<String>) -> Result<Deck, String> {
    let mut deck = Deck::new(state.next_deck_id());
    let deck_name = match name {
        Some(name) if !name.trim().is_empty() => name,
        _ => "Untitled".to_string(),
    };
    deck.set_name(deck_name);
    deck.recount_game_changers();
    state.decks.write().unwrap().push(deck.clone());
    state.save_deck(&deck)?;
    Ok(deck) //RETURNING THE DECK SO NEW DECK TILE CAN PULL FROM IT RATHER THAN RERENDER ALL THE DECKS IN UI
}
#[tauri::command]
pub fn rename_deck(
    state: State<'_, AppState>,
    deck_id: u64,
    name: Option<String>,
) -> Result<Deck, String> {
    let deck_name = match name {
        Some(name) if !name.trim().is_empty() => name,
        _ => "Untitled".to_string(),
    };

    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;
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
pub async fn get_deck(
    state: State<'_, AppState>,
    app: AppHandle,
    deck_id: u64,
) -> Result<Deck, String> {
    let deck = state
        .decks
        .read()
        .map_err(|_| "Failed to acquire deck lock".to_string())?
        .iter()
        .find(|deck| deck.id() == deck_id)
        .cloned()
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    // Trigger image fetching for this specific deck in background
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(
            app_handle,
            Some(deck_id),
            None,
            None,
            None,
        )
        .await;
    });

    Ok(deck)
}

// TEST COMMAND
#[tauri::command]
pub fn test_command(greeting: Option<String>) -> String {
    greeting.unwrap_or("greeting".to_string())
}

#[tauri::command]
pub fn delete_deck(state: State<'_, AppState>, deck_id: u64) -> Result<(), String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

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
pub async fn duplicate_deck(
    state: State<'_, AppState>,
    app: AppHandle,
    deck_id: u64,
) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let source_deck = decks
        .iter()
        .find(|deck| deck.id() == deck_id)
        .cloned()
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    let mut duplicated_deck = source_deck;
    duplicated_deck.set_id(state.next_deck_id());
    let base_name = duplicated_deck.get_name().to_string();
    duplicated_deck.set_name(format!("{} (Copy)", base_name));
    duplicated_deck.recount_game_changers();

    decks.push(duplicated_deck.clone());
    drop(decks);
    state.save_deck(&duplicated_deck)?;

    // Trigger image fetching in background for the new deck
    let app_handle = app.clone();
    let new_deck_id = duplicated_deck.id();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(
            app_handle,
            Some(new_deck_id),
            None,
            None,
            None,
        )
        .await;
    });

    Ok(duplicated_deck)
}

#[tauri::command]
pub async fn add_card_to_deck(
    state: State<'_, AppState>,
    app: AppHandle,
    deck_id: u64,
    name: String,
) -> Result<Deck, String> {
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
    deck.recount_game_changers();
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;

    // Trigger image fetching in background for this specific deck
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(
            app_handle,
            Some(deck_id),
            None,
            None,
            None,
        )
        .await;
    });

    Ok(updated)
}

#[tauri::command]
pub async fn bulk_add_cards_to_deck(
    state: State<'_, AppState>,
    app: AppHandle,
    deck_id: u64,
    cards: Vec<(u32, String)>,
) -> Result<Deck, String> {
    if cards.is_empty() {
        return Err("No cards to add".to_string());
    }

    let mut new_cards = Vec::new();
    for (qty, name) in cards {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }

        for _ in 0..qty {
            let card = card_from_db_by_name(trimmed, state.next_card_id())?
                .ok_or_else(|| format!("Card '{}' not found in local database", trimmed))?;
            new_cards.push(card);
        }
    }

    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    for card in new_cards {
        deck.add_card(card);
    }
    deck.recount_game_changers();
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;

    // Trigger image fetching in background ONCE after all cards are added to the deck
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(
            app_handle,
            Some(deck_id),
            None,
            None,
            None,
        )
        .await;
    });

    Ok(updated)
}

#[tauri::command]
pub fn get_packages(state: State<'_, AppState>) -> Vec<Package> {
    state.packages.read().unwrap().clone()
}

#[tauri::command]
pub async fn get_package(
    state: State<'_, AppState>,
    app: AppHandle,
    package_id: u64,
) -> Result<Package, String> {
    let package = state
        .packages
        .read()
        .map_err(|_| "Failed to acquire package lock".to_string())?
        .iter()
        .find(|package| package.id() == package_id)
        .cloned()
        .ok_or_else(|| format!("Package with id {} not found", package_id))?;

    // Trigger image fetching for this specific package in background
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(
            app_handle,
            None,
            Some(package_id),
            None,
            None,
        )
        .await;
    });

    Ok(package)
}

#[tauri::command]
pub fn create_package(state: State<'_, AppState>, name: Option<String>) -> Result<Package, String> {
    let package_name = match name {
        Some(name) if !name.trim().is_empty() => name.trim().to_string(),
        _ => "Untitled Package".to_string(),
    };

    let package = Package::new(state.next_package_id(), package_name);
    state.packages.write().unwrap().push(package.clone());
    state.save_package(&package)?;
    Ok(package)
}

#[tauri::command]
pub async fn bulk_add_cards_to_package(
    state: State<'_, AppState>,
    app: AppHandle,
    package_id: u64,
    cards: Vec<(u32, String)>,
) -> Result<Package, String> {
    if cards.is_empty() {
        return Err("No cards to add".to_string());
    }

    let mut new_cards = Vec::new();
    for (qty, name) in cards {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }

        for _ in 0..qty {
            let card = card_from_db_by_name(trimmed, state.next_card_id())?
                .ok_or_else(|| format!("Card '{}' not found in local database", trimmed))?;
            new_cards.push(card);
        }
    }

    let mut packages = state
        .packages
        .write()
        .map_err(|_| "Failed to acquire package lock".to_string())?;

    let package = packages
        .iter_mut()
        .find(|p| p.id() == package_id)
        .ok_or_else(|| format!("Package with id {} not found", package_id))?;

    for card in new_cards {
        package.add_card(card);
    }

    let updated = package.clone();
    drop(packages);

    state.save_package(&updated)?;

    // Trigger image fetching in background ONCE after all cards are added to the package
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(
            app_handle,
            None,
            Some(package_id),
            None,
            None,
        )
        .await;
    });

    Ok(updated)
}

#[tauri::command]
pub async fn add_card_to_package(
    state: State<'_, AppState>,
    app: AppHandle,
    package_id: u64,
    card_name: String,
) -> Result<Package, String> {
    let trimmed = card_name.trim();
    if trimmed.is_empty() {
        return Err("Card name cannot be empty".to_string());
    }

    let new_card = card_from_db_by_name(trimmed, state.next_card_id())?
        .ok_or_else(|| format!("Card '{trimmed}' not found in local database"))?;

    let mut packages = state
        .packages
        .write()
        .map_err(|_| "Failed to acquire package lock".to_string())?;
    let package = packages
        .iter_mut()
        .find(|package| package.id() == package_id)
        .ok_or_else(|| format!("Package with id {} not found", package_id))?;

    package.add_card(new_card);
    let updated = package.clone();
    drop(packages);

    state.save_package(&updated)?;

    // Trigger image fetching in background for this specific package
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(
            app_handle,
            None,
            Some(package_id),
            None,
            None,
        )
        .await;
    });

    Ok(updated)
}

#[tauri::command]
pub fn rename_package(
    state: State<'_, AppState>,
    package_id: u64,
    name: Option<String>,
) -> Result<Package, String> {
    let package_name = match name {
        Some(name) if !name.trim().is_empty() => name.trim().to_string(),
        _ => "Untitled Package".to_string(),
    };

    let mut packages = state
        .packages
        .write()
        .map_err(|_| "Failed to acquire package lock".to_string())?;
    let package = packages
        .iter_mut()
        .find(|package| package.id() == package_id)
        .ok_or_else(|| format!("Package with id {} not found", package_id))?;

    package.set_name(package_name);
    let updated = package.clone();
    drop(packages);

    state.save_package(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn set_package_description(
    state: State<'_, AppState>,
    package_id: u64,
    description: Option<String>,
) -> Result<Package, String> {
    let package_description = description.unwrap_or_default().trim().to_string();

    let mut packages = state
        .packages
        .write()
        .map_err(|_| "Failed to acquire package lock".to_string())?;
    let package = packages
        .iter_mut()
        .find(|package| package.id() == package_id)
        .ok_or_else(|| format!("Package with id {} not found", package_id))?;

    package.set_description(package_description);
    let updated = package.clone();
    drop(packages);

    state.save_package(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_package(state: State<'_, AppState>, package_id: u64) -> Result<(), String> {
    let mut packages = state
        .packages
        .write()
        .map_err(|_| "Failed to acquire package lock".to_string())?;

    let index = packages
        .iter()
        .position(|package| package.id() == package_id)
        .ok_or_else(|| format!("Package with id {} not found", package_id))?;

    packages.remove(index);
    drop(packages);
    state.delete_package(package_id)?;
    Ok(())
}

#[tauri::command]
pub async fn duplicate_package(
    state: State<'_, AppState>,
    app: AppHandle,
    package_id: u64,
) -> Result<Package, String> {
    let mut packages = state
        .packages
        .write()
        .map_err(|_| "Failed to acquire package lock".to_string())?;

    let source_package = packages
        .iter()
        .find(|package| package.id() == package_id)
        .cloned()
        .ok_or_else(|| format!("Package with id {} not found", package_id))?;

    let mut duplicated_package = source_package;
    let base_name = duplicated_package.get_name().to_string();
    duplicated_package.set_id(state.next_package_id());
    duplicated_package.set_name(format!("{} (Copy)", base_name));

    let source_cards = duplicated_package.get_cards().to_vec();
    let mut copied_package = Package::new(
        duplicated_package.id(),
        duplicated_package.get_name().to_string(),
    );

    for mut card in source_cards {
        card.set_id(state.next_card_id());
        copied_package.add_card(card);
    }

    packages.push(copied_package.clone());
    drop(packages);
    state.save_package(&copied_package)?;

    // Trigger image fetching in background for the new package
    let app_handle = app.clone();
    let new_pkg_id = copied_package.id();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(
            app_handle,
            None,
            Some(new_pkg_id),
            None,
            None,
        )
        .await;
    });

    Ok(copied_package)
}

#[tauri::command]
pub async fn add_package_to_deck(
    state: State<'_, AppState>,
    app: AppHandle,
    deck_id: u64,
    package_id: u64,
) -> Result<Deck, String> {
    let package = state
        .packages
        .read()
        .map_err(|_| "Failed to acquire package lock".to_string())?
        .iter()
        .find(|package| package.id() == package_id)
        .cloned()
        .ok_or_else(|| format!("Package with id {} not found", package_id))?;

    let mut cards_to_add = Vec::new();
    for card in package.get_cards() {
        let mut cloned_card = card.clone();
        cloned_card.set_id(state.next_card_id());
        cards_to_add.push(cloned_card);
    }

    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;
    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    for card in cards_to_add {
        deck.add_card(card);
    }
    deck.recount_game_changers();
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;

    // Trigger image fetching in background for this specific deck
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::image_commands::fetch_card_images(
            app_handle,
            Some(deck_id),
            None,
            None,
            None,
        )
        .await;
    });

    Ok(updated)
}

#[tauri::command]
pub fn remove_card_from_package(
    state: State<'_, AppState>,
    package_id: u64,
    card_id: u64,
) -> Result<Package, String> {
    let mut packages = state
        .packages
        .write()
        .map_err(|_| "Failed to acquire package lock".to_string())?;

    let package = packages
        .iter_mut()
        .find(|package| package.id() == package_id)
        .ok_or_else(|| format!("Package with id {} not found", package_id))?;

    let card_index = package
        .get_cards()
        .iter()
        .position(|card| card.id() == card_id)
        .ok_or_else(|| {
            format!(
                "Card with id {} not found in package {}",
                card_id, package_id
            )
        })?;

    if !package.remove_card_at(card_index) {
        return Err(format!(
            "Failed to remove card with id {} from package {}",
            card_id, package_id
        ));
    }

    let updated = package.clone();
    drop(packages);

    state.save_package(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn remove_card_from_deck(
    state: State<'_, AppState>,
    deck_id: u64,
    card_id: u64,
) -> Result<Deck, String> {
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
        return Err(format!(
            "Failed to remove card with id {} from deck {}",
            card_id, deck_id
        ));
    }

    deck.recount_game_changers();
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn set_deck_commander(
    state: State<'_, AppState>,
    deck_id: u64,
    card_id: u64,
) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.set_single_commander_from_deck(card_id)?;
    deck.recount_game_changers();
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn set_deck_partner(
    state: State<'_, AppState>,
    deck_id: u64,
    card_id: u64,
) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.set_partner_commander_from_deck(card_id)?;
    deck.recount_game_changers();
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn remove_deck_partner(
    state: State<'_, AppState>,
    deck_id: u64,
    card_id: u64,
) -> Result<Deck, String> {
    let mut decks = state
        .decks
        .write()
        .map_err(|_| "Failed to acquire deck lock".to_string())?;

    let deck = decks
        .iter_mut()
        .find(|deck| deck.id() == deck_id)
        .ok_or_else(|| format!("Deck with id {} not found", deck_id))?;

    deck.remove_partner_commander_from_deck(card_id)?;
    deck.recount_game_changers();
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
    deck.recount_game_changers();
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
    deck.recount_game_changers();
    let updated = deck.clone();
    drop(decks);

    state.save_deck(&updated)?;
    Ok(updated)
}
