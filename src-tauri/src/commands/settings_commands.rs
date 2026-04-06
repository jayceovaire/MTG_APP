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
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    match update {
        Some(update) => Ok(UpdateCheckResponse {
            available: true,
            current_version: update.current_version.clone(),
            version: Some(update.version.clone()),
            body: update.body.clone(),
            date: update.date.map(|date| date.to_string()),
        }),
        None => Ok(UpdateCheckResponse {
            available: false,
            current_version: app.package_info().version.to_string(),
            version: None,
            body: None,
            date: None,
        }),
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<String, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    match update {
        Some(update) => {
            let version = update.version.clone();
            update
                .download_and_install(|_, _| {}, || {})
                .await
                .map_err(|e| e.to_string())?;
            app.request_restart();
            Ok(format!("Installed update {version}. Restarting application."))
        }
        None => Ok("No updates available.".to_string()),
    }
}
