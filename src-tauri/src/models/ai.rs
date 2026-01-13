//! AI-related models for command suggestions
//!
//! This module contains all data structures used for AI provider configuration,
//! settings, and command suggestion requests/responses.

use serde::{Deserialize, Serialize};

/// Supported AI provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AIProviderType {
    OpenAI,
    Anthropic,
    Gemini,
    Ollama,
    Custom,
}

impl Default for AIProviderType {
    fn default() -> Self {
        AIProviderType::OpenAI
    }
}

/// AI Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIProviderConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub provider_type: AIProviderType,
    pub name: String,
    /// Encrypted API key (stored encrypted, decrypted at runtime)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    pub model: String,
    pub is_enabled: bool,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Trigger mode for AI suggestions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AITriggerMode {
    Auto,
    Manual,
    Both,
}

impl Default for AITriggerMode {
    fn default() -> Self {
        AITriggerMode::Both
    }
}

/// Global AI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AISettings {
    pub is_enabled: bool,
    pub trigger_mode: AITriggerMode,
    pub auto_trigger_delay_ms: u32,
    pub max_suggestions: u8,
    pub include_history: bool,
    pub include_saved_commands: bool,
    pub include_cwd: bool,
    pub include_system_info: bool,
}

impl Default for AISettings {
    fn default() -> Self {
        AISettings {
            is_enabled: false,
            trigger_mode: AITriggerMode::Both,
            auto_trigger_delay_ms: 500,
            max_suggestions: 5,
            include_history: true,
            include_saved_commands: true,
            include_cwd: true,
            include_system_info: true,
        }
    }
}

/// Terminal context for AI suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIContext {
    pub current_input: String,
    #[serde(default)]
    pub recent_commands: Vec<String>,
    #[serde(default)]
    pub saved_commands: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
}

/// Individual AI suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AISuggestion {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

/// Request for AI suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AISuggestionRequest {
    pub provider_id: String,
    pub context: AIContext,
}

/// Response from AI suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AISuggestionResponse {
    pub suggestions: Vec<AISuggestion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
}

/// Result of testing AI provider connection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIConnectionTestResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

/// Request to create a new AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAIProviderRequest {
    #[serde(rename = "type")]
    pub provider_type: AIProviderType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    pub model: String,
    #[serde(default)]
    pub is_default: bool,
}

/// Request to update an AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAIProviderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

/// Default base URLs for providers
pub fn get_default_base_url(provider_type: &AIProviderType) -> &'static str {
    match provider_type {
        AIProviderType::OpenAI => "https://api.openai.com/v1",
        AIProviderType::Anthropic => "https://api.anthropic.com/v1",
        AIProviderType::Gemini => "https://generativelanguage.googleapis.com/v1beta",
        AIProviderType::Ollama => "http://localhost:11434",
        AIProviderType::Custom => "",
    }
}

/// Default models for providers
pub fn get_default_model(provider_type: &AIProviderType) -> &'static str {
    match provider_type {
        AIProviderType::OpenAI => "gpt-4o-mini",
        AIProviderType::Anthropic => "claude-3-5-haiku-latest",
        AIProviderType::Gemini => "gemini-2.0-flash",
        AIProviderType::Ollama => "llama3.2",
        AIProviderType::Custom => "gpt-3.5-turbo",
    }
}

/// Get available models for a provider type
pub fn get_available_models(provider_type: &AIProviderType) -> Vec<&'static str> {
    match provider_type {
        AIProviderType::OpenAI => vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-4",
            "gpt-3.5-turbo",
        ],
        AIProviderType::Anthropic => vec![
            "claude-3-5-sonnet-latest",
            "claude-3-5-haiku-latest",
            "claude-3-opus-latest",
        ],
        AIProviderType::Gemini => vec!["gemini-2.0-flash", "gemini-1.5-flash", "gemini-1.5-pro"],
        AIProviderType::Ollama => vec![
            "llama3.2",
            "llama3.1",
            "mistral",
            "codellama",
            "deepseek-coder",
        ],
        AIProviderType::Custom => vec!["gpt-3.5-turbo"],
    }
}
