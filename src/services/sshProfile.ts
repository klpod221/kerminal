import { invoke } from "@tauri-apps/api/core";
import type { SSHProfile } from "../types/ssh";

/**
 * Create a new SSH profile
 * @param profile - SSH profile configuration
 * @returns The created SSH profile
 */
export async function createSSHProfile(profile: SSHProfile) {
  return await invoke("create_ssh_profile", { ...profile });
}


