use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResponse {
    pub available: bool,
    pub current_version: String,
    pub version: Option<String>,
    pub body: Option<String>,
    pub date: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateCheckResponse, String> {
    println!(
        "[updater] checking for updates from app version {}",
        app.package_info().version
    );

    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = match updater.check().await {
        Ok(update) => update,
        Err(error) => {
            eprintln!("[updater] update check failed: {error}");
            return Err(error.to_string());
        }
    };

    match update {
        Some(update) => {
            println!(
                "[updater] update available: current={}, latest={}",
                update.current_version, update.version
            );
            Ok(UpdateCheckResponse {
                available: true,
                current_version: update.current_version.clone(),
                version: Some(update.version.clone()),
                body: update.body.clone(),
                date: update.date.map(|date| date.to_string()),
            })
        }
        None => {
            println!(
                "[updater] no update available; current version remains {}",
                app.package_info().version
            );
            Ok(UpdateCheckResponse {
                available: false,
                current_version: app.package_info().version.to_string(),
                version: None,
                body: None,
                date: None,
            })
        }
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<String, String> {
    println!("[updater] install_update invoked");
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = match updater.check().await {
        Ok(update) => update,
        Err(error) => {
            eprintln!("[updater] update re-check failed before install: {error}");
            return Err(error.to_string());
        }
    };

    match update {
        Some(update) => {
            let version = update.version.clone();
            println!("[updater] downloading and installing version {version}");
            update
                .download_and_install(|_, _| {}, || {})
                .await
                .map_err(|e| {
                    eprintln!("[updater] download/install failed: {e}");
                    e.to_string()
                })?;
            println!("[updater] install finished; requesting restart");
            app.request_restart();
            Ok(format!("Installed update {version}. Restarting application."))
        }
        None => {
            println!("[updater] install requested but no update was available");
            Ok("No updates available.".to_string())
        }
    }
}
