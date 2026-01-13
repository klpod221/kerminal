import { invoke } from "@tauri-apps/api/core";
import type {
  AIProviderConfig,
  AIContext,
  AISuggestionResponse,
  AIConnectionTestResult,
  AISettings,
  AISuggestionRequest,
  AIProviderType,
  CreateAIProviderRequest,
  UpdateAIProviderRequest,
} from "../types/ai";

/**
 * AI Service
 * Handles interaction with the Rust backend for AI functionality
 */
export const aiService = {
  /**
   * Get global AI settings
   */
  async getSettings(): Promise<AISettings> {
    return invoke("get_ai_settings");
  },

  /**
   * Update global AI settings
   */
  async updateSettings(settings: AISettings): Promise<void> {
    await invoke("update_ai_settings", { settings });
  },

  /**
   * Get all configured AI providers
   */
  async getProviders(): Promise<AIProviderConfig[]> {
    return invoke("get_ai_providers");
  },

  /**
   * Add a new AI provider
   */
  async addProvider(
    request: CreateAIProviderRequest
  ): Promise<AIProviderConfig> {
    return invoke("add_ai_provider", { request });
  },

  /**
   * Update an existing AI provider
   */
  async updateProvider(
    id: string,
    request: UpdateAIProviderRequest
  ): Promise<AIProviderConfig> {
    return invoke("update_ai_provider", { id, request });
  },

  /**
   * Remove an AI provider
   */
  async removeProvider(id: string): Promise<void> {
    await invoke("remove_ai_provider", { id });
  },

  /**
   * Test connection to an AI provider
   */
  async testConnection(
    config: AIProviderConfig
  ): Promise<AIConnectionTestResult> {
    return invoke("test_ai_connection", { config });
  },

  /**
   * Get command suggestions
   */
  async getSuggestions(
    request: AISuggestionRequest
  ): Promise<AISuggestionResponse> {
    return invoke("get_ai_suggestions", { request });
  },

  /**
   * Get available models for a provider type (Static helper)
   */
  getAvailableModels(type: AIProviderType): string[] {
    switch (type) {
      case "openai":
        return [
          "gpt-4o",
          "gpt-4o-mini",
          "gpt-4-turbo",
          "gpt-4",
          "gpt-3.5-turbo",
        ];
      case "anthropic":
        return [
          "claude-3-5-sonnet-latest",
          "claude-3-5-haiku-latest",
          "claude-3-opus-latest",
        ];
      case "gemini":
        return ["gemini-2.0-flash", "gemini-1.5-flash", "gemini-1.5-pro"];
      case "ollama":
        return [
          "llama3.2",
          "llama3.1",
          "mistral",
          "codellama",
          "deepseek-coder",
        ];
      case "custom":
        return ["gpt-3.5-turbo"];
      default:
        return [];
    }
  },
};
