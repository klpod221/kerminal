import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  AIProviderConfig,
  AISettings,
  AIContext,
  AISuggestionResponse,
  AIConnectionTestResult,
  CreateAIProviderRequest,
  UpdateAIProviderRequest,
} from "../types/ai";
import { aiService } from "../services/ai";
import { handleError, type ErrorContext } from "../utils/errorHandler";
import { message } from "../utils/message";

/**
 * AI Store
 * Manages AI providers, settings, and suggestion requests via Backend Service
 */
export const useAIStore = defineStore("ai", () => {
  // State
  const providers = ref<AIProviderConfig[]>([]);
  const settings = ref<AISettings>({
    isEnabled: false,
    triggerMode: "both",
    autoTriggerDelayMs: 500,
    maxSuggestions: 5,
    includeHistory: true,
    includeSavedCommands: true,
    includeCwd: true,
    includeSystemInfo: true,
  });
  const isLoading = ref(false);
  const lastSuggestions = ref<AISuggestionResponse | null>(null);

  // Computed
  const defaultProvider = computed(() =>
    providers.value.find((p) => p.isDefault && p.isEnabled)
  );

  const enabledProviders = computed(() =>
    providers.value.filter((p) => p.isEnabled)
  );

  const hasProviders = computed(() => providers.value.length > 0);

  const isAIEnabled = computed(
    () => settings.value.isEnabled && defaultProvider.value !== undefined
  );

  /**
   * Load settings and providers from Backend
   */
  const loadData = async (): Promise<void> => {
    const context: ErrorContext = {
      operation: "Load AI Data",
    };

    try {
      isLoading.value = true;
      const [backendSettings, backendProviders] = await Promise.all([
        aiService.getSettings(),
        aiService.getProviders(),
      ]);

      settings.value = backendSettings;
      providers.value = backendProviders;
    } catch (error) {
      const errorMessage = handleError(error, context);
      console.error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Add a new AI provider
   */
  const addProvider = async (
    request: CreateAIProviderRequest
  ): Promise<AIProviderConfig> => {
    try {
      const newProvider = await aiService.addProvider(request);
      await loadData(); // Reload to ensure sync
      message.success(`AI provider "${request.name}" added successfully`);
      return newProvider;
    } catch (error) {
      message.error("Failed to add provider");
      throw error;
    }
  };

  /**
   * Update an existing AI provider
   */
  const updateProvider = async (
    id: string,
    request: UpdateAIProviderRequest
  ): Promise<void> => {
    try {
      await aiService.updateProvider(id, request);
      await loadData();
      message.success("AI provider updated");
    } catch (error) {
      message.error("Failed to update provider");
      throw error;
    }
  };

  /**
   * Remove an AI provider
   */
  const removeProvider = async (id: string): Promise<void> => {
    try {
      await aiService.removeProvider(id);
      await loadData();
      message.success("AI provider removed");
    } catch (error) {
      message.error("Failed to remove provider");
      throw error;
    }
  };

  /**
   * Set default provider
   */
  const setDefaultProvider = async (id: string): Promise<void> => {
    // We update the specific provider to be default.
    // Backend handles unsetting others.
    await updateProvider(id, { isDefault: true });
  };

  /**
   * Update AI settings
   */
  const updateSettings = async (
    newSettings: Partial<AISettings>
  ): Promise<void> => {
    try {
      const merged = { ...settings.value, ...newSettings };
      await aiService.updateSettings(merged);
      settings.value = merged;
    } catch (error) {
      message.error("Failed to update settings");
      throw error;
    }
  };

  /**
   * Test connection to a provider
   */
  const testConnection = async (
    provider: AIProviderConfig
  ): Promise<AIConnectionTestResult> => {
    isLoading.value = true;
    try {
      return await aiService.testConnection(provider);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Get AI suggestions for current context
   */
  const getSuggestions = async (
    context: AIContext
  ): Promise<AISuggestionResponse | null> => {
    if (!isAIEnabled.value || !defaultProvider.value) {
      return null;
    }

    // Don't fetch if input is too short
    if (context.currentInput.trim().length < 2) {
      return null;
    }

    const opContext: ErrorContext = {
      operation: "Get AI Suggestions",
    };

    try {
      isLoading.value = true;
      const response = await aiService.getSuggestions({
        providerId: defaultProvider.value.id,
        context,
      });
      lastSuggestions.value = response;
      return response;
    } catch (error) {
      const errorMessage = handleError(error, opContext);
      console.error(errorMessage);
      return null;
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Clear cached suggestions
   */
  const clearSuggestions = (): void => {
    lastSuggestions.value = null;
  };

  /**
   * Get provider by ID
   */
  const getProviderById = (id: string): AIProviderConfig | undefined => {
    return providers.value.find((p) => p.id === id);
  };

  /**
   * Get available models for a provider type
   */
  const getAvailableModels = aiService.getAvailableModels;

  // Initialize on store creation
  loadData();

  return {
    // State
    providers,
    settings,
    isLoading,
    lastSuggestions,

    // Computed
    defaultProvider,
    enabledProviders,
    hasProviders,
    isAIEnabled,

    // Actions
    loadSettings: loadData, // Alias for compatibility
    addProvider,
    updateProvider,
    removeProvider,
    setDefaultProvider,
    updateSettings,
    testConnection,
    getSuggestions,
    clearSuggestions,
    getProviderById,
    getAvailableModels,
  };
});
