use crate::settings::{get_settings, write_settings};
use tauri::AppHandle;

#[tauri::command]
pub fn set_mistral_api_key(app: AppHandle, api_key: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.mistral_api_key = if api_key.is_empty() {
        None
    } else {
        Some(api_key)
    };
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
pub fn get_mistral_api_key(app: AppHandle) -> Result<Option<String>, String> {
    let settings = get_settings(&app);
    Ok(settings.mistral_api_key)
}

#[tauri::command]
pub fn has_mistral_api_key(app: AppHandle) -> Result<bool, String> {
    let settings = get_settings(&app);
    Ok(settings.mistral_api_key.is_some())
}

#[tauri::command]
pub fn set_deepgram_api_key(app: AppHandle, api_key: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.deepgram_api_key = if api_key.is_empty() {
        None
    } else {
        Some(api_key)
    };
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
pub fn get_deepgram_api_key(app: AppHandle) -> Result<Option<String>, String> {
    let settings = get_settings(&app);
    Ok(settings.deepgram_api_key)
}

#[tauri::command]
pub fn has_deepgram_api_key(app: AppHandle) -> Result<bool, String> {
    let settings = get_settings(&app);
    Ok(settings.deepgram_api_key.is_some())
}

#[tauri::command]
pub fn set_assemblyai_api_key(app: AppHandle, api_key: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.assemblyai_api_key = if api_key.is_empty() {
        None
    } else {
        Some(api_key)
    };
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
pub fn get_assemblyai_api_key(app: AppHandle) -> Result<Option<String>, String> {
    let settings = get_settings(&app);
    Ok(settings.assemblyai_api_key)
}

#[tauri::command]
pub fn has_assemblyai_api_key(app: AppHandle) -> Result<bool, String> {
    let settings = get_settings(&app);
    Ok(settings.assemblyai_api_key.is_some())
}
