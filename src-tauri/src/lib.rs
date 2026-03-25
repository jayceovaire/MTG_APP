use crate::state::AppState;
use tauri::Manager;

pub mod commands;
pub mod models;
pub mod state;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = app.state::<AppState>();
            state
                .initialize_persistence(app.handle())
                .map_err(|e| -> Box<dyn std::error::Error> { std::io::Error::other(e).into() })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::collection_commands::add_card_to_collection,
            commands::collection_commands::duplicate_collection_card,
            commands::collection_commands::remove_collection_card,    
            commands::collection_commands::get_collection,
            commands::collection_commands::search_card_suggestions,
            commands::deck_commands::create_deck,
            commands::deck_commands::get_decks,
            commands::deck_commands::get_deck,
            commands::deck_commands::rename_deck,
            commands::deck_commands::delete_deck,
            commands::deck_commands::duplicate_deck,
            commands::deck_commands::add_card_to_deck,
            commands::deck_commands::remove_card_from_deck,
            commands::deck_commands::set_deck_commander,
            commands::deck_commands::set_deck_partner,
            commands::deck_commands::remove_deck_partner,
            commands::deck_commands::remove_deck_commander,
            commands::deck_commands::delete_deck_commander,
            commands::deck_commands::test_command,
        ])
        .manage(app_state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
