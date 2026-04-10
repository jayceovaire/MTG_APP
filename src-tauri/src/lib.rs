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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let state = app.state::<AppState>();
            state
                .initialize_persistence(app.handle())
                .map_err(|e| -> Box<dyn std::error::Error> { std::io::Error::other(e).into() })?;

            commands::collection_commands::initialize_db_paths(app.handle())
                .map_err(|e| -> Box<dyn std::error::Error> { std::io::Error::other(e).into() })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::collection_commands::get_collection,
            commands::collection_commands::get_card,
            commands::collection_commands::get_random_card,
            commands::collection_commands::set_collection_card_favorite,
            commands::collection_commands::search_card_suggestions,
            commands::collection_commands::add_card_to_collection,
            commands::collection_commands::bulk_add_cards_to_collection,
            commands::collection_commands::duplicate_collection_card,
            commands::collection_commands::remove_collection_card,
            commands::deck_commands::create_deck,
            commands::deck_commands::get_decks,
            commands::deck_commands::get_deck,
            commands::deck_commands::rename_deck,
            commands::deck_commands::delete_deck,
            commands::deck_commands::duplicate_deck,
            commands::deck_commands::reorder_decks,
            commands::deck_commands::add_card_to_deck,
            commands::deck_commands::bulk_add_cards_to_deck,
            commands::deck_commands::get_packages,
            commands::deck_commands::get_package,
            commands::deck_commands::create_package,
            commands::deck_commands::rename_package,
            commands::deck_commands::set_package_description,
            commands::deck_commands::delete_package,
            commands::deck_commands::duplicate_package,
            commands::deck_commands::reorder_packages,
            commands::deck_commands::add_card_to_package,
            commands::deck_commands::bulk_add_cards_to_package,
            commands::deck_commands::add_package_to_deck,
            commands::deck_commands::remove_card_from_package,
            commands::deck_commands::remove_card_from_deck,
            commands::deck_commands::set_deck_commander,
            commands::deck_commands::set_deck_partner,
            commands::deck_commands::remove_deck_partner,
            commands::deck_commands::remove_deck_commander,
            commands::deck_commands::delete_deck_commander,
            commands::deck_commands::test_command,
            commands::image_commands::fetch_card_images,
            commands::image_commands::get_base64_images,
            commands::image_commands::get_most_recent_cached_image,
            commands::crispi_commands::evaluate_deck_roles,
            commands::settings_commands::check_for_updates,
            commands::settings_commands::install_update,
        ])
        .manage(app_state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
