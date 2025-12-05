import { defineStore } from "pinia";
import { ref } from "vue";
import type {
  TerminalProfile,
  CreateTerminalProfileRequest,
  UpdateTerminalProfileRequest,
} from "../types/terminalProfile";
import { invoke } from "@tauri-apps/api/core";
import { message } from "../utils/message";
import { handleError, type ErrorContext } from "../utils/errorHandler";

export const useTerminalProfileStore = defineStore("terminalProfile", () => {
  const profiles = ref<TerminalProfile[]>([]);
  const isLoading = ref(false);

  // Load from Backend
  const loadProfiles = async () => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Load Terminal Profiles",
    };

    try {
      const stored = await invoke<TerminalProfile[]>("list_terminal_profiles");
      profiles.value = stored;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
    } finally {
      isLoading.value = false;
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
      const newProfile = await invoke<TerminalProfile>("create_terminal_profile", { request });
      profiles.value.push(newProfile);
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
      const updatedProfile = await invoke<TerminalProfile>("update_terminal_profile", { id, request });
      const index = profiles.value.findIndex((p) => p.id === id);
      if (index !== -1) {
        profiles.value[index] = updatedProfile;
        return updatedProfile;
      }
      // If not found in local list but updated successfully, reload list
      await loadProfiles();
      return updatedProfile;
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
      await invoke("delete_terminal_profile", { id });
      const index = profiles.value.findIndex((p) => p.id === id);
      if (index !== -1) {
        profiles.value.splice(index, 1);
      }
      return true;
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

