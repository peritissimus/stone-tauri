//! Settings Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::ports::outbound::Setting,
};

/// Response for get_all_settings
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAllSettingsResponse {
    pub settings: Vec<Setting>,
}

/// Response for get_setting
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSettingResponse {
    pub value: Option<String>,
}

#[tauri::command]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<GetSettingResponse, String> {
    let value = state
        .settings_usecases
        .get(&key)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GetSettingResponse { value })
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    state
        .settings_usecases
        .set(&key, &value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_settings(
    state: State<'_, AppState>,
) -> Result<GetAllSettingsResponse, String> {
    let settings = state
        .settings_usecases
        .get_all()
        .await
        .map_err(|e| e.to_string())?;

    Ok(GetAllSettingsResponse { settings })
}
