/**
 * AI Provider Types
 * Types for AI-powered command suggestions with multiple provider support
 */

/**
 * Supported AI provider types
 */
export type AIProviderType =
  | "openai"
  | "anthropic"
  | "gemini"
  | "ollama"
  | "custom";

/**
 * AI Provider configuration
 */
export interface AIProviderConfig {
  id: string;
  type: AIProviderType;
  name: string;
  apiKey?: string; // Encrypted with master password
  baseUrl?: string; // Custom endpoint (required for ollama/custom)
  model: string;
  isEnabled: boolean;
  isDefault: boolean;
  createdAt: string;
  updatedAt: string;
}

/**
 * AI suggestion trigger mode
 */
export type AITriggerMode = "auto" | "manual" | "both";

/**
 * AI global settings
 */
export interface AISettings {
  isEnabled: boolean;
  triggerMode: AITriggerMode;
  autoTriggerDelayMs: number; // Delay before auto-trigger (default: 500ms)
  maxSuggestions: number; // Max suggestions to show (default: 5)
  includeHistory: boolean; // Include command history in context
  includeSavedCommands: boolean; // Include saved commands in context
  includeCwd: boolean; // Include current working directory
  includeSystemInfo: boolean; // Include OS/shell info
}

/**
 * Default AI settings
 */
export const DEFAULT_AI_SETTINGS: AISettings = {
  isEnabled: false,
  triggerMode: "both",
  autoTriggerDelayMs: 500,
  maxSuggestions: 5,
  includeHistory: true,
  includeSavedCommands: true,
  includeCwd: true,
  includeSystemInfo: true,
};

/**
 * Default models for each provider
 */
export const DEFAULT_MODELS: Record<AIProviderType, string> = {
  openai: "gpt-4o-mini",
  anthropic: "claude-3-haiku-20240307",
  gemini: "gemini-2.0-flash",
  ollama: "llama3",
  custom: "",
};

/**
 * Default base URLs for providers
 */
export const DEFAULT_BASE_URLS: Record<AIProviderType, string> = {
  openai: "https://api.openai.com/v1",
  anthropic: "https://api.anthropic.com/v1",
  gemini: "https://generativelanguage.googleapis.com/v1beta",
  ollama: "http://localhost:11434",
  custom: "",
};

/**
 * AI command suggestion
 */
export interface AISuggestion {
  command: string;
  description: string;
  confidence: number; // 0-1 confidence score
}

/**
 * Context sent to AI for suggestions
 */
export interface AIContext {
  currentInput: string;
  recentCommands?: string[];
  savedCommands?: string[];
  cwd?: string;
  shell?: string;
  os?: string;
}

/**
 * AI suggestion request
 */
export interface AISuggestionRequest {
  providerId: string;
  context: AIContext;
}

/**
 * AI suggestion response
 */
export interface AISuggestionResponse {
  suggestions: AISuggestion[];
  providerId: string;
  latencyMs: number;
}

/**
 * AI provider connection test result
 */
export interface AIConnectionTestResult {
  success: boolean;
  message: string;
  latencyMs?: number;
}

/**
 * Create AI provider request
 */
export interface CreateAIProviderRequest {
  type: AIProviderType;
  name: string;
  apiKey?: string;
  baseUrl?: string;
  model: string;
  isDefault?: boolean;
}

/**
 * Update AI provider request
 */
export interface UpdateAIProviderRequest {
  name?: string;
  apiKey?: string;
  baseUrl?: string;
  model?: string;
  isEnabled?: boolean;
  isDefault?: boolean;
}
