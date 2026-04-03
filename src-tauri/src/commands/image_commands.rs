use crate::state::AppState;
use crate::models::card_model::Card;
use crate::models::deck_model::CommanderSelection;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{Manager, AppHandle, Emitter};
use serde::Deserialize;
use tokio::fs as tfs;
use tokio::io::AsyncWriteExt;

use std::time::SystemTime;
use once_cell::sync::Lazy;
use tokio::sync::Semaphore;

static SCRYFALL_SEMAPHORE: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(1));

#[derive(Deserialize)]
struct ScryfallCard {
    id: Option<String>,
    image_uris: Option<ScryfallImageUris>,
    card_faces: Option<Vec<ScryfallCardFace>>,
}

#[derive(Deserialize)]
struct ScryfallCardFace {
    image_uris: Option<ScryfallImageUris>,
}

#[derive(Deserialize)]
struct ScryfallImageUris {
    png: Option<String>,
}

#[tauri::command]
pub async fn fetch_card_images(
    app: AppHandle,
    deck_id: Option<u64>,
    package_id: Option<u64>,
    collection: Option<bool>,
    all: Option<bool>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut any_changed = false;
    let cache_dir = app.path().local_data_dir()
        .map_err(|e| format!("Failed to resolve local app data directory: {e}"))?
        .join("mtg_app")
        .join("card_images");

    let is_all = all.unwrap_or(false);

    println!("Starting image fetch. Cache dir: {:?}, target: deck={:?}, package={:?}, collection={:?}, all={}", 
        cache_dir, deck_id, package_id, collection, is_all);

    if !cache_dir.exists() {
        println!("Creating cache directory: {:?}", cache_dir);
        fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create cache directory: {e}"))?;
    }

    let client = reqwest::Client::builder()
        .user_agent("MTG_App/0.1.0 (contact: support@mtg_app.local)")
        .build()
        .map_err(|e| format!("Failed to create reqwest client: {e}"))?;

    // 1. Process Collection
    if collection.unwrap_or(false) || is_all {
        let collection_ids: Vec<u64> = state.collection.read()
            .map_err(|_| "Failed to lock collection for reading")?
            .iter()
            .map(|c| c.id())
            .collect();

        if !collection_ids.is_empty() {
            println!("Processing {} collection cards", collection_ids.len());
        }

        for id in collection_ids {
            let card_opt = state.collection.read()
                .map_err(|_| "Failed to lock collection for card lookup")?
                .iter()
                .find(|c| c.id() == id)
                .cloned();

            if let Some(mut card) = card_opt {
                match process_card(&mut card, &cache_dir, &client).await {
                    Ok(true) => {
                        any_changed = true;
                        let mut collection_lock = state.collection.write()
                            .map_err(|_| "Failed to lock collection for update")?;
                        if let Some(c) = collection_lock.iter_mut().find(|c| c.id() == id) {
                            *c = card;
                            let _ = state.save_collection_card(c);
                        }
                    }
                    Ok(false) => {}
                    Err(e) => println!("Error processing card {}: {}", card.get_name(), e),
                }
            }
        }
    }

    // 2. Process Decks
    let decks_to_process: Vec<u64> = if let Some(id) = deck_id {
        vec![id]
    } else if is_all {
        state.decks.read()
            .map_err(|_| "Failed to lock decks for reading")?
            .iter()
            .map(|d| d.id())
            .collect()
    } else {
        Vec::new()
    };

    if !decks_to_process.is_empty() {
        println!("Processing {} decks", decks_to_process.len());
    }

    for d_id in decks_to_process {
        let deck_opt = state.decks.read()
            .map_err(|_| "Failed to lock decks for deck lookup")?
            .iter()
            .find(|d| d.id() == d_id)
            .cloned();

        if let Some(mut deck) = deck_opt {
            let mut changed = false;

            // Cards in deck
            for card in deck.get_cards_mut().iter_mut() {
                match process_card(card, &cache_dir, &client).await {
                    Ok(true) => changed = true,
                    Ok(false) => {}
                    Err(e) => println!("Error processing card in deck: {}", e),
                }
            }

            // Commander
            match deck.get_commander_mut() {
                CommanderSelection::None => {}
                CommanderSelection::Single(card) => {
                    match process_card(card, &cache_dir, &client).await {
                        Ok(true) => changed = true,
                        Ok(false) => {}
                        Err(e) => println!("Error processing commander: {}", e),
                    }
                }
                CommanderSelection::Partner(c1, c2) => {
                    match process_card(c1, &cache_dir, &client).await {
                        Ok(true) => changed = true,
                        Ok(false) => {}
                        Err(e) => println!("Error processing partner: {}", e),
                    }
                    match process_card(c2, &cache_dir, &client).await {
                        Ok(true) => changed = true,
                        Ok(false) => {}
                        Err(e) => println!("Error processing partner: {}", e),
                    }
                }
            }

            if changed {
                any_changed = true;
                let mut decks = state.decks.write()
                    .map_err(|_| "Failed to lock decks for update")?;
                if let Some(d) = decks.iter_mut().find(|d| d.id() == d_id) {
                    *d = deck;
                    let _ = state.save_deck(d);
                }
            }
        }
    }

    // 3. Process Packages
    let packages_to_process: Vec<u64> = if let Some(id) = package_id {
        vec![id]
    } else if is_all {
        state.packages.read()
            .map_err(|_| "Failed to lock packages for reading")?
            .iter()
            .map(|p| p.id())
            .collect()
    } else {
        Vec::new()
    };

    if !packages_to_process.is_empty() {
        println!("Processing {} packages", packages_to_process.len());
    }

    for p_id in packages_to_process {
        let package_opt = state.packages.read()
            .map_err(|_| "Failed to lock packages for package lookup")?
            .iter()
            .find(|p| p.id() == p_id)
            .cloned();

        if let Some(mut package) = package_opt {
            let mut changed = false;
            for card in package.get_cards_mut().iter_mut() {
                match process_card(card, &cache_dir, &client).await {
                    Ok(true) => changed = true,
                    Ok(false) => {}
                    Err(e) => println!("Error processing card in package: {}", e),
                }
            }
            if changed {
                any_changed = true;
                let mut packages = state.packages.write()
                    .map_err(|_| "Failed to lock packages for update")?;
                if let Some(p) = packages.iter_mut().find(|p| p.id() == p_id) {
                    *p = package;
                    let _ = state.save_package(p);
                }
            }
        }
    }

    println!("Image fetch complete.");
    if any_changed {
        let _ = app.emit("images-updated", ());
    }
    Ok(())
}

pub(crate) async fn process_card(card: &mut Card, cache_dir: &Path, client: &reqwest::Client) -> Result<bool, String> {
    // Check if we already have the image path and if it exists
    if !card.get_image().is_empty() {
        let path = PathBuf::from(card.get_image());
        if path.exists() {
            return Ok(false);
        }
    }

    let extension = "png";
    let filename = if let Some(scryfall_id) = card.scryfall_id() {
        format!("{}.{}", scryfall_id, extension)
    } else {
        format!("{}.{}", card.id(), extension)
    };
    let dest_path = cache_dir.join(filename);

    if dest_path.exists() {
        // println!("Found existing image in cache for {}", card.get_name());
        card.set_image(dest_path.to_string_lossy().to_string());
        return Ok(true);
    }

    println!("Fetching image for {} (ID: {})", card.get_name(), card.id());

    // Try to get PNG URL
    let png_url = {
        // Respect Scryfall API rate limit globally
        let _permit = SCRYFALL_SEMAPHORE.acquire().await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Priority 1: Use Card Name as per JS example (it's more direct for some reason?)
        // Priority 2: Use Scryfall ID if name fails or ID is present
        let api_url = format!("https://api.scryfall.com/cards/named?exact={}", urlencoding::encode(card.get_name()));
        
        if let Some(_scryfall_id) = card.scryfall_id() {
             // If we have an ID, we could also use that, but let's try name first if it's there
             // unless name has weird characters that Scryfall doesn't like.
             // Actually, let's stick to name first to match the user's example.
        }

        println!("Querying card data: {}", api_url);
        let response = client.get(&api_url).send().await
            .map_err(|e| format!("Failed to fetch card data from Scryfall ({}): {}", api_url, e))?;
        
        if !response.status().is_success() {
            // If name lookup fails, try ID lookup if available
            if let Some(scryfall_id) = card.scryfall_id() {
                let id_url = format!("https://api.scryfall.com/cards/{}", scryfall_id);
                println!("Name lookup failed, trying ID lookup: {}", id_url);
                // Respect Scryfall API rate limit for the second lookup as well
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let response = client.get(&id_url).send().await
                    .map_err(|e| format!("Failed to fetch card data from Scryfall ({}): {}", id_url, e))?;
                
                if response.status().is_success() {
                    let scryfall_card: ScryfallCard = response.json().await
                        .map_err(|e| format!("Failed to parse Scryfall response: {}", e))?;
                    
                    if let Some(sid) = &scryfall_card.id {
                        card.set_scryfall_id(sid.clone());
                    }

                    extract_png_url(&scryfall_card)
                } else {
                    println!("ID lookup also failed for {}: {}", card.get_name(), response.status());
                    None
                }
            } else {
                println!("Name lookup failed for {}: {}", card.get_name(), response.status());
                None
            }
        } else {
            let scryfall_card: ScryfallCard = response.json().await
                .map_err(|e| format!("Failed to parse Scryfall response: {}", e))?;
            
            if let Some(sid) = &scryfall_card.id {
                card.set_scryfall_id(sid.clone());
            }

            extract_png_url(&scryfall_card)
        }
    };

    if let Some(uri) = png_url {
        // If we acquired a Scryfall ID during lookup, re-calculate the path
        // to use the content-based filename.
        let final_dest_path = if let Some(sid) = card.scryfall_id() {
            cache_dir.join(format!("{}.{}", sid, extension))
        } else {
            dest_path.clone()
        };

        if final_dest_path.exists() {
            println!("Found image at {:?} after API lookup, skipping download for {}", final_dest_path, card.get_name());
            card.set_image(final_dest_path.to_string_lossy().to_string());
            return Ok(true);
        }

        println!("Downloading PNG from: {}", uri);
        // Respect Scryfall API rate limit globally for the image fetch as well
        let _permit = SCRYFALL_SEMAPHORE.acquire().await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let response = client.get(&uri).send().await
            .map_err(|e| format!("Failed to fetch image from {}: {}", uri, e))?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to download image from {}: {}", uri, response.status()));
        }

        let bytes = response.bytes().await.map_err(|e| format!("Failed to read image bytes: {e}"))?;
        let mut file = tfs::File::create(&final_dest_path).await.map_err(|e| format!("Failed to create file: {e}"))?;
        file.write_all(&bytes).await.map_err(|e| format!("Failed to write to file: {e}"))?;

        card.set_image(final_dest_path.to_string_lossy().to_string());
        println!("Successfully saved image for {} to {:?}", card.get_name(), final_dest_path);
        return Ok(true);
    }

    println!("Could not find PNG URL for {}", card.get_name());
    Ok(false)
}

fn extract_png_url(scryfall_card: &ScryfallCard) -> Option<String> {
    if let Some(uris) = &scryfall_card.image_uris {
        uris.png.clone()
    } else if let Some(faces) = &scryfall_card.card_faces {
        faces.get(0).and_then(|f| f.image_uris.as_ref()).and_then(|u| u.png.clone())
    } else {
        None
    }
}

#[tauri::command]
pub async fn get_base64_images(paths: Vec<String>) -> Result<Vec<String>, String> {
    let mut results = Vec::with_capacity(paths.len());
    for path in paths {
        if path.is_empty() {
            results.push(String::new());
            continue;
        }
        match fs::read(&path) {
            Ok(bytes) => {
                let b64 = b64_encode(&bytes);
                results.push(format!("data:image/png;base64,{}", b64));
            }
            Err(_) => results.push(String::new()),
        }
    }
    Ok(results)
}

#[tauri::command]
pub async fn get_most_recent_cached_image(app: AppHandle) -> Result<Option<String>, String> {
    let cache_dir = app.path().local_data_dir()
        .map_err(|e| format!("Failed to resolve local app data directory: {e}"))?
        .join("mtg_app")
        .join("card_images");

    if !cache_dir.exists() {
        return Ok(None);
    }

    let mut most_recent_file: Option<(PathBuf, SystemTime)> = None;

    for entry in fs::read_dir(cache_dir).map_err(|e| format!("Failed to read cache directory: {e}"))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let path = entry.path();

        if path.is_file() {
            let metadata = entry.metadata().map_err(|e| format!("Failed to read metadata: {e}"))?;
            let modified = metadata.modified().map_err(|e| format!("Failed to read modified time: {e}"))?;

            match most_recent_file {
                None => most_recent_file = Some((path, modified)),
                Some((_, ref current_most_recent_time)) => {
                    if modified > *current_most_recent_time {
                        most_recent_file = Some((path, modified));
                    }
                }
            }
        }
    }

    if let Some((path, _)) = most_recent_file {
        let bytes = fs::read(&path).map_err(|e| format!("Failed to read image file: {e}"))?;
        let base64_image = b64_encode(&bytes);
        Ok(Some(format!("data:image/png;base64,{}", base64_image)))
    } else {
        Ok(None)
    }
}

pub(crate) fn b64_encode(input: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    let mut it = input.chunks_exact(3);
    for chunk in it.by_ref() {
        result.push(CHARSET[(chunk[0] >> 2) as usize] as char);
        result.push(CHARSET[((chunk[0] & 0x03) << 4 | chunk[1] >> 4) as usize] as char);
        result.push(CHARSET[((chunk[1] & 0x0f) << 2 | chunk[2] >> 6) as usize] as char);
        result.push(CHARSET[(chunk[2] & 0x3f) as usize] as char);
    }
    let rem = it.remainder();
    if rem.len() == 1 {
        result.push(CHARSET[(rem[0] >> 2) as usize] as char);
        result.push(CHARSET[((rem[0] & 0x03) << 4) as usize] as char);
        result.push('=');
        result.push('=');
    } else if rem.len() == 2 {
        result.push(CHARSET[(rem[0] >> 2) as usize] as char);
        result.push(CHARSET[((rem[0] & 0x03) << 4 | rem[1] >> 4) as usize] as char);
        result.push(CHARSET[((rem[1] & 0x0f) << 2) as usize] as char);
        result.push('=');
    }
    result
}
