import { api } from "./api";
import type { SSHProfile } from "../types/ssh";

/**
 * Create a new SSH profile
 * @param profile - SSH profile configuration
 * @returns The created SSH profile
 */
export async function createSSHProfile(profile: SSHProfile) {
  return await api.call("create_ssh_profile", profile);
}


