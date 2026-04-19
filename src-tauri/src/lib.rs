use crate::state::AppState;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

pub mod commands;
pub mod models;
pub mod state;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { .. } => {
                    let state = window.state::<AppState>();
                    let app_handle = window.app_handle();
                    let windows = app_handle.webview_windows();
                    
                    if window.label() == "main" {
                         let mut sidecar_child = state.sidecar_child.write().unwrap();
                         if let Some(child) = sidecar_child.take() {
                             let _ = child.kill();
                             println!("Sidecar process killed because main window is closing.");
                         }
                    } else {
                        // For non-main windows, only kill if it's the last one
                        // We count EXCEPT the one currently being closed
                        let other_windows_count = windows.values().filter(|w| w.label() != window.label()).count();
                        if other_windows_count == 0 {
                            let mut sidecar_child = state.sidecar_child.write().unwrap();
                            if let Some(child) = sidecar_child.take() {
                                let _ = child.kill();
                                println!("Sidecar process killed on last window closure ({}).", window.label());
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .setup(|app| {
            // Kill any existing sidecar instances to ensure we start fresh with correct data and avoid duplicates
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/IM", "mtg-sidecar-x86_64-pc-windows-msvc.exe", "/T"])
                    .output();
                // Brief sleep to allow port to be released
                std::thread::sleep(std::time::Duration::from_millis(200));
            }

            let resource_dir = app.path().resource_dir().expect("failed to get resource dir");
            let app_data_dir = app.path().local_data_dir().expect("failed to get app data dir").join("mtg_app");
            let _ = std::fs::create_dir_all(&app_data_dir);
            let index_db_path = app_data_dir.join("sidecar_index.db");
            
            let db_dir = resource_dir.join("src/db");
            let scryfall_path = db_dir.join("scryfall.db");

            let sidecar_command = app.shell().sidecar("mtg-sidecar").unwrap()
                .args(["--runtime-dir", resource_dir.join("src/db").to_str().unwrap()])
                .args(["--db-path", scryfall_path.to_str().unwrap()])
                .args(["--index-db-path", index_db_path.to_str().unwrap()]);
            
            let (mut rx, child) = sidecar_command
                .spawn()
                .expect("Failed to spawn sidecar");

            let state = app.state::<AppState>();
            if let Ok(mut sidecar_child) = state.sidecar_child.write() {
                *sidecar_child = Some(child);
            }

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match event {
                        tauri_plugin_shell::process::CommandEvent::Stdout(line_bytes) => {
                            let line = String::from_utf8_lossy(&line_bytes);
                            println!("Sidecar: {}", line);
                        }
                        tauri_plugin_shell::process::CommandEvent::Stderr(line_bytes) => {
                            let line = String::from_utf8_lossy(&line_bytes);
                            if line.contains("[INFO]") || line.contains("Serving on") {
                                println!("Sidecar: {}", line);
                            } else {
                                eprintln!("Sidecar Error: {}", line);
                            }
                        }
                        tauri_plugin_shell::process::CommandEvent::Terminated(payload) => {
                            let status_str = match payload.code {
                                Some(code) => format!("code {}", code),
                                None => "signal or unknown".to_string(),
                            };
                            println!("Sidecar terminated with status {}", status_str);
                            let state = app_handle.state::<AppState>();
                            if let Ok(mut sidecar_child) = state.sidecar_child.write() {
                                *sidecar_child = None;
                            };
                            // If the sidecar terminated unexpectedly, we might want to exit the app or log it
                            println!("Sidecar process lost. Application might need restart for full functionality.");
                        }
                        _ => {}
                    }
                }
            });

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
            commands::sidecar_commands::is_sidecar_running,
            commands::sidecar_commands::get_sidecar_index,
            commands::sidecar_commands::get_sidecar_variants,
            commands::sidecar_commands::submit_combo_to_sidecar,
            commands::sidecar_commands::kill_sidecar,
        ])
        .manage(app_state)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. } => {
                    let state = app_handle.state::<AppState>();
                    let mut sidecar_child = state.sidecar_child.write().unwrap();
                    if let Some(child) = sidecar_child.take() {
                        let _ = child.kill();
                        println!("Sidecar process killed during exit.");
                    }
                }
                _ => {}
            }
        });
}
