// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { ref } from "vue";
import { connectSFTP, disconnectSFTP } from "../../services/sftp";
import { useSFTPStore } from "../../stores/sftp";

export function useSftpSession() {
  const sftpStore = useSFTPStore();
  const sessionId = ref<string | null>(null);
  const homeDir = ref<string>("/");
  const currentProfileId = ref<string | null>(null);
  const connecting = ref<boolean>(false);
  const error = ref<string | null>(null);

  async function connect(profileId: string): Promise<boolean> {
    if (connecting.value) return false;
    connecting.value = true;
    error.value = null;

    try {
      if (sessionId.value) {
        await disconnect();
      }

      const res = await connectSFTP(profileId);
      sessionId.value = res.sessionId;
      sftpStore.activeSessionId = res.sessionId;
      homeDir.value = res.homeDir || "/";
      currentProfileId.value = profileId;
      return true;
    } catch (e: any) {
      error.value = e?.message || String(e);
      return false;
    } finally {
      connecting.value = false;
    }
  }

  async function disconnect(): Promise<void> {
    if (!sessionId.value) return;
    try {
      await disconnectSFTP(sessionId.value);
    } catch (e) {
      console.error("[useSftpSession] Failed to disconnect:", e);
    } finally {
      sessionId.value = null;
      sftpStore.activeSessionId = null;
      currentProfileId.value = null;
    }
  }

  return {
    sessionId,
    homeDir,
    currentProfileId,
    connecting,
    error,
    connect,
    disconnect,
  };
}
