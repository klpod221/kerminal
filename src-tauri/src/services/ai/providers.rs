use super::super::super::models::ai::{
    AIContext, AIProviderConfig, AISuggestion, AISuggestionResponse,
};
use crate::error::AppError;
use async_trait::async_trait;
use serde_json::json;
use std::time::Instant;

const SYSTEM_PROMPT: &str = r#"You are an intelligent terminal command assistant.
Your goal is to suggest the most appropriate terminal command based on the user's input and context.
You must return ONLY a JSON response in the following format:
{
  "suggestions": [
    {
      "command": "actual command here",
      "description": "brief explanation of what this does",
      "confidence": 0.0 to 1.0 (optional)
    }
  ]
}

Rules:
1. Provide valid, safe terminal commands.
2. If the user's input is a natural language description, translate it to a command.
3. If the user's input is a partial command, complete it.
4. Use the provided context (cwd, history, saved commands, system info) to make better suggestions.
5. Do NOT interpret or execute the command.
6. Return ONLY valid JSON. No markdown formatting, no explanations outside the JSON."#;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn get_suggestions(
        &self,
        config: &AIProviderConfig,
        context: &AIContext,
    ) -> Result<AISuggestionResponse, AppError>;

    async fn test_connection(&self, config: &AIProviderConfig) -> Result<bool, AppError>;
}

// --- OpenAI Provider ---

pub struct OpenAIProvider;

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn get_suggestions(
        &self,
        config: &AIProviderConfig,
        context: &AIContext,
    ) -> Result<AISuggestionResponse, AppError> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::config_error("OpenAI API key is missing"))?;

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");

        // Use decrypted key logic here if/when encryption is fully integrated?
        // For now we assume the service decrypts it before calling this, OR
        // we decrypt it here. The prompt in `service/mod.rs` suggests this is the raw impl.
        // Let's assume the passed config has the usable key (potentially decrypted by the caller).

        let client = reqwest::Client::new();
        let start = Instant::now();

        let prompt = build_user_prompt(context);

        let response = client
            .post(format!(
                "{}/chat/completions",
                base_url.trim_end_matches('/')
            ))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": config.model,
                "messages": [
                    { "role": "system", "content": SYSTEM_PROMPT },
                    { "role": "user", "content": prompt }
                ],
                "temperature": 0.3,
                "response_format": { "type": "json_object" }
            }))
            .send()
            .await
            .map_err(|e| AppError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            return Err(AppError::external_api(format!(
                "OpenAI error: {}",
                error_text
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::serialization_error(e.to_string()))?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| AppError::external_api("Invalid response format from OpenAI"))?;

        let parsed: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            AppError::serialization_error(format!("Failed to parse JSON response: {}", e))
        })?;

        let suggestions: Vec<AISuggestion> =
            serde_json::from_value(parsed["suggestions"].clone()).unwrap_or_default();

        Ok(AISuggestionResponse {
            suggestions,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            provider_id: Some(config.id.clone()),
        })
    }

    async fn test_connection(&self, config: &AIProviderConfig) -> Result<bool, AppError> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::config_error("OpenAI API key is missing"))?;

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/models", base_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| AppError::network(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

// --- Anthropic Provider ---

pub struct AnthropicProvider;

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn get_suggestions(
        &self,
        config: &AIProviderConfig,
        context: &AIContext,
    ) -> Result<AISuggestionResponse, AppError> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::config_error("Anthropic API key is missing"))?;

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com/v1");

        let client = reqwest::Client::new();
        let start = Instant::now();
        let prompt = build_user_prompt(context);

        let response = client
            .post(format!("{}/messages", base_url.trim_end_matches('/')))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": config.model,
                "system": SYSTEM_PROMPT,
                "messages": [
                    { "role": "user", "content": prompt }
                ],
                "max_tokens": 1024,
                "temperature": 0.3
            }))
            .send()
            .await
            .map_err(|e| AppError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            return Err(AppError::external_api(format!(
                "Anthropic error: {}",
                error_text
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::serialization_error(e.to_string()))?;

        let content = body["content"][0]["text"]
            .as_str()
            .ok_or_else(|| AppError::external_api("Invalid response format from Anthropic"))?;

        // Anthropic doesn't enforce JSON mode as heavily as OpenAI sometimes, so we might need to find the JSON
        let json_str = extract_json(content)?;

        let parsed: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
            AppError::serialization_error(format!("Failed to parse JSON response: {}", e))
        })?;

        let suggestions: Vec<AISuggestion> =
            serde_json::from_value(parsed["suggestions"].clone()).unwrap_or_default();

        Ok(AISuggestionResponse {
            suggestions,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            provider_id: Some(config.id.clone()),
        })
    }

    async fn test_connection(&self, config: &AIProviderConfig) -> Result<bool, AppError> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::config_error("Anthropic API key is missing"))?;

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com/v1");

        let client = reqwest::Client::new();
        // Anthropic doesn't have a simple "list models" endpoint that is free/easy properly without messages?
        // Actually it does, /v1/models is available but might require beta headers.
        // Easiest test is sending a dummy message with max_tokens 1.

        let response = client
            .post(format!("{}/messages", base_url.trim_end_matches('/')))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": config.model,
                "messages": [
                    { "role": "user", "content": "Hi" }
                ],
                "max_tokens": 1
            }))
            .send()
            .await
            .map_err(|e| AppError::network(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

// --- Gemini Provider ---

pub struct GeminiProvider;

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn get_suggestions(
        &self,
        config: &AIProviderConfig,
        context: &AIContext,
    ) -> Result<AISuggestionResponse, AppError> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::config_error("Gemini API key is missing"))?;

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com/v1beta");

        let client = reqwest::Client::new();
        let start = Instant::now();
        let prompt = build_user_prompt(context);

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            base_url.trim_end_matches('/'),
            config.model,
            api_key
        );

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "contents": [{
                    "parts": [{ "text": format!("{}\n\nUser Input: {}", SYSTEM_PROMPT, prompt) }]
                }],
                "generationConfig": {
                    "response_mime_type": "application/json"
                }
            }))
            .send()
            .await
            .map_err(|e| AppError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            return Err(AppError::external_api(format!(
                "Gemini error: {}",
                error_text
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::serialization_error(e.to_string()))?;

        let content = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| AppError::external_api("Invalid response format from Gemini"))?;

        let parsed: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            AppError::serialization_error(format!("Failed to parse JSON response: {}", e))
        })?;

        let suggestions: Vec<AISuggestion> =
            serde_json::from_value(parsed["suggestions"].clone()).unwrap_or_default();

        Ok(AISuggestionResponse {
            suggestions,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            provider_id: Some(config.id.clone()),
        })
    }

    async fn test_connection(&self, config: &AIProviderConfig) -> Result<bool, AppError> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::config_error("Gemini API key is missing"))?;

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com/v1beta");

        let client = reqwest::Client::new();
        let url = format!(
            "{}/models/{}?key={}",
            base_url.trim_end_matches('/'),
            config.model,
            api_key
        );

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::network(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

// --- Ollama Provider ---

pub struct OllamaProvider;

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn get_suggestions(
        &self,
        config: &AIProviderConfig,
        context: &AIContext,
    ) -> Result<AISuggestionResponse, AppError> {
        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("http://localhost:11434");

        let client = reqwest::Client::new();
        let start = Instant::now();
        let prompt = build_user_prompt(context);

        let response = client
            .post(format!("{}/api/generate", base_url.trim_end_matches('/')))
            .json(&json!({
                "model": config.model,
                "system": SYSTEM_PROMPT,
                "prompt": prompt,
                "stream": false,
                "format": "json"
            }))
            .send()
            .await
            .map_err(|e| AppError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            return Err(AppError::external_api(format!(
                "Ollama error: {}",
                error_text
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::serialization_error(e.to_string()))?;

        let content = body["response"]
            .as_str()
            .ok_or_else(|| AppError::external_api("Invalid response format from Ollama"))?;

        let parsed: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            AppError::serialization_error(format!("Failed to parse JSON response: {}", e))
        })?;

        let suggestions: Vec<AISuggestion> =
            serde_json::from_value(parsed["suggestions"].clone()).unwrap_or_default();

        Ok(AISuggestionResponse {
            suggestions,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            provider_id: Some(config.id.clone()),
        })
    }

    async fn test_connection(&self, config: &AIProviderConfig) -> Result<bool, AppError> {
        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("http://localhost:11434");

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/tags", base_url.trim_end_matches('/')))
            .send()
            .await
            .map_err(|e| AppError::network(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

// --- Common Helpers ---

fn build_user_prompt(context: &AIContext) -> String {
    let mut prompt = String::new();

    // System Info
    if let (Some(os), Some(shell)) = (&context.os, &context.shell) {
        prompt.push_str(&format!("OS: {}\nShell: {}\n", os, shell));
    }

    // CWD
    if let Some(cwd) = &context.cwd {
        prompt.push_str(&format!("Working Directory: {}\n", cwd));
    }

    // Recent History
    if !context.recent_commands.is_empty() {
        prompt.push_str("Recent Commands:\n");
        for cmd in context.recent_commands.iter().rev().take(10) {
            prompt.push_str(&format!("- {}\n", cmd));
        }
    }

    // Saved Commands (optional - might be too long, limit it)
    if !context.saved_commands.is_empty() {
        prompt.push_str("Saved/Favorite Commands:\n");
        for cmd in context.saved_commands.iter().take(20) {
            prompt.push_str(&format!("- {}\n", cmd));
        }
    }

    // Current Input
    prompt.push_str(&format!("\nCurrent Input: {}\n", context.current_input));

    prompt
}

fn extract_json(text: &str) -> Result<String, AppError> {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if start <= end {
                return Ok(text[start..=end].to_string());
            }
        }
    }
    Err(AppError::serialization_error("No JSON found in response"))
}
