use crate::error::AppError;
use crate::models::ai::{
    AIConnectionTestResult, AIProviderConfig, AISettings, AISuggestionRequest,
    AISuggestionResponse, CreateAIProviderRequest, UpdateAIProviderRequest,
};
use crate::state::AppState;

#[tauri::command]
pub async fn get_ai_settings(state: tauri::State<'_, AppState>) -> Result<AISettings, AppError> {
    Ok(state.ai_service.get_settings().await)
}

#[tauri::command]
pub async fn update_ai_settings(
    state: tauri::State<'_, AppState>,
    settings: AISettings,
) -> Result<(), AppError> {
    state.ai_service.update_settings(settings).await
}

#[tauri::command]
pub async fn get_ai_providers(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AIProviderConfig>, AppError> {
    Ok(state.ai_service.get_providers().await)
}

#[tauri::command]
pub async fn add_ai_provider(
    state: tauri::State<'_, AppState>,
    request: CreateAIProviderRequest,
) -> Result<AIProviderConfig, AppError> {
    state.ai_service.add_provider(request).await
}

#[tauri::command]
pub async fn update_ai_provider(
    state: tauri::State<'_, AppState>,
    id: String,
    request: UpdateAIProviderRequest,
) -> Result<AIProviderConfig, AppError> {
    state.ai_service.update_provider(id, request).await
}

#[tauri::command]
pub async fn remove_ai_provider(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    state.ai_service.remove_provider(id).await
}

#[tauri::command]
pub async fn test_ai_connection(
    state: tauri::State<'_, AppState>,
    config: AIProviderConfig,
) -> Result<AIConnectionTestResult, AppError> {
    state.ai_service.test_connection(config).await
}

#[tauri::command]
pub async fn get_ai_suggestions(
    state: tauri::State<'_, AppState>,
    request: AISuggestionRequest,
) -> Result<AISuggestionResponse, AppError> {
    state.ai_service.get_suggestions(request).await
}
