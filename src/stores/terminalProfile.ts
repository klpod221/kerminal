import { defineStore } from "pinia";
import { ref } from "vue";
import type {
  TerminalProfile,
  CreateTerminalProfileRequest,
  UpdateTerminalProfileRequest,
} from "../types/terminalProfile";
import { v4 as uuidv4 } from "uuid";
import { Store } from "@tauri-apps/plugin-store";
import { message } from "../utils/message";
import { handleError, type ErrorContext } from "../utils/errorHandler";

// Tauri store instance
let store: Store | null = null;

// Initialize Tauri store
const initStore = async () => {
  if (!store) {
    store = await Store.load("terminal-profiles.json");
  }
  return store;
};

export const useTerminalProfileStore = defineStore("terminalProfile", () => {
  const profiles = ref<TerminalProfile[]>([]);
  const isLoading = ref(false);

  // Load from Tauri store
  const loadProfiles = async () => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Load Terminal Profiles",
    };

    try {
      const storeInstance = await initStore();
      const stored = await storeInstance.get<TerminalProfile[]>("profiles");
      if (stored) {
        profiles.value = stored;
      }
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  // Save to Tauri store
  const saveProfiles = async () => {
    const context: ErrorContext = {
      operation: "Save Terminal Profiles",
    };

    try {
      const storeInstance = await initStore();
      await storeInstance.set("profiles", profiles.value);
      await storeInstance.save();
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
    }
  };

  // Initialize
  loadProfiles();

  const createProfile = async (
    request: CreateTerminalProfileRequest,
  ): Promise<TerminalProfile> => {
    const context: ErrorContext = {
      operation: "Create Terminal Profile",
      context: { name: request.name },
    };

    try {
      const newProfile: TerminalProfile = {
        id: uuidv4(),
        ...request,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };
      profiles.value.push(newProfile);
      await saveProfiles();
      return newProfile;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  };

  const updateProfile = async (
    id: string,
    request: UpdateTerminalProfileRequest,
  ): Promise<TerminalProfile | undefined> => {
    const context: ErrorContext = {
      operation: "Update Terminal Profile",
      context: { profileId: id },
    };

    try {
      const index = profiles.value.findIndex((p) => p.id === id);
      if (index !== -1) {
        profiles.value[index] = {
          ...profiles.value[index],
          ...request,
          updatedAt: Date.now(),
        };
        await saveProfiles();
        return profiles.value[index];
      }
      return undefined;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  };

  const deleteProfile = async (id: string): Promise<boolean> => {
    const context: ErrorContext = {
      operation: "Delete Terminal Profile",
      context: { profileId: id },
    };

    try {
      const index = profiles.value.findIndex((p) => p.id === id);
      if (index !== -1) {
        profiles.value.splice(index, 1);
        await saveProfiles();
        return true;
      }
      return false;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  };

  const getProfile = (id: string): TerminalProfile | undefined => {
    return profiles.value.find((p) => p.id === id);
  };

  return {
    profiles,
    isLoading,
    createProfile,
    updateProfile,
    deleteProfile,
    getProfile,
    loadProfiles,
  };
});
