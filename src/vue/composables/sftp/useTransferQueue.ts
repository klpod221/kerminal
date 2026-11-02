// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { ref, onMounted, onUnmounted, computed } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { TransferProgress } from "../../types/sftp";
import {
  getAllTransfers,
  pauseSFTPTransfer,
  resumeSFTPTransfer,
  cancelSFTPTransfer,
  retryTransfer as retryTransferApi,
  setTransferPriority as setPriorityApi,
  uploadSFTPFile,
  downloadSFTPFile,
  clearCompletedTransfers,
} from "../../services/sftp";

export function useTransferQueue() {
  const transfers = ref<Map<string, TransferProgress>>(new Map());
  const loading = ref<boolean>(false);
  const isDrawerOpen = ref<boolean>(false);

  let unlistenProgress: UnlistenFn | null = null;
  let unlistenComplete: UnlistenFn | null = null;
  let unlistenError: UnlistenFn | null = null;
  let pollInterval: any = null;

  async function refreshTransfers() {
    try {
      loading.value = true;
      const list = await getAllTransfers();
      const map = new Map<string, TransferProgress>();
      for (const t of list) {
        map.set(t.transferId, t);
      }
      transfers.value = map;
    } catch (e) {
      console.error("[useTransferQueue] Failed to refresh transfers:", e);
    } finally {
      loading.value = false;
    }
  }

  function startPolling(intervalMs = 2000) {
    if (pollInterval) clearInterval(pollInterval);
    pollInterval = setInterval(refreshTransfers, intervalMs);
  }

  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }

  async function setupListeners() {
    unlistenProgress = await listen<any>("sftp_transfer_progress", (event) => {
      const { transferId, transferredBytes, totalBytes, speed } = event.payload;
      const existing = transfers.value.get(transferId);
      if (existing) {
        existing.transferredBytes = transferredBytes;
        if (totalBytes) existing.totalBytes = totalBytes;
        if (speed) existing.speedBytesPerSec = speed;
      } else {
        refreshTransfers();
      }
    });

    unlistenComplete = await listen<any>("sftp_transfer_complete", (event) => {
      const { transferId } = event.payload;
      const existing = transfers.value.get(transferId);
      if (existing) {
        existing.status = "completed";
      }
      refreshTransfers();
    });

    unlistenError = await listen<any>("sftp_transfer_error", (event) => {
      const { transferId, error } = event.payload;
      const existing = transfers.value.get(transferId);
      if (existing) {
        existing.status = "failed";
        existing.error = error;
      }
      refreshTransfers();
    });
  }

  async function pause(transferId: string) {
    try {
      await pauseSFTPTransfer(transferId);
      const t = transfers.value.get(transferId);
      if (t) t.status = "paused";
    } catch (e) {
      console.error("Failed to pause transfer:", e);
    }
  }

  async function resume(transferId: string) {
    try {
      await resumeSFTPTransfer(transferId);
      const t = transfers.value.get(transferId);
      if (t) t.status = "queued";
    } catch (e) {
      console.error("Failed to resume transfer:", e);
    }
  }

  async function cancel(transferId: string) {
    try {
      await cancelSFTPTransfer(transferId);
      const t = transfers.value.get(transferId);
      if (t) t.status = "cancelled";
    } catch (e) {
      console.error("Failed to cancel transfer:", e);
    }
  }

  async function retry(transferId: string) {
    try {
      await retryTransferApi(transferId);
      const t = transfers.value.get(transferId);
      if (t) t.status = "queued";
    } catch (e) {
      console.error("Failed to retry transfer:", e);
    }
  }

  async function setPriority(transferId: string, priority: number) {
    try {
      await setPriorityApi(transferId, priority);
      const t = transfers.value.get(transferId);
      if (t) t.priority = priority;
    } catch (e) {
      console.error("Failed to set priority:", e);
    }
  }

  async function pauseAll() {
    for (const t of activeTransfers.value) {
      await pause(t.transferId);
    }
  }

  async function resumeAll() {
    for (const t of transferList.value.filter((x) => x.status === "paused")) {
      await resume(t.transferId);
    }
  }

  async function clearCompleted() {
    try {
      await clearCompletedTransfers();
    } catch (e) {
      console.error("[useTransferQueue] Failed to clear completed transfers on backend:", e);
    }
    for (const [id, t] of transfers.value.entries()) {
      if (t.status === "completed" || t.status === "cancelled") {
        transfers.value.delete(id);
      }
    }
  }

  async function addUploadTask(
    sessionId: string,
    localPath: string,
    remotePath: string,
  ): Promise<string | null> {
    try {
      const id = await uploadSFTPFile(sessionId, localPath, remotePath);
      await refreshTransfers();
      return id;
    } catch (e) {
      console.error("[useTransferQueue] Failed to add upload task:", e);
      return null;
    }
  }

  async function addDownloadTask(
    sessionId: string,
    remotePath: string,
    localPath: string,
  ): Promise<string | null> {
    try {
      const id = await downloadSFTPFile(sessionId, remotePath, localPath);
      await refreshTransfers();
      return id;
    } catch (e) {
      console.error("[useTransferQueue] Failed to add download task:", e);
      return null;
    }
  }

  const transferList = computed(() => Array.from(transfers.value.values()));

  const activeTransfers = computed(() =>
    transferList.value.filter(
      (t) =>
        t.status === "inprogress" ||
        (t.status as string) === "in_progress" ||
        (t.status as string) === "running" ||
        t.status === "queued",
    ),
  );

  const activeCount = computed(() => activeTransfers.value.length);

  onMounted(() => {
    refreshTransfers();
    setupListeners();
  });

  onUnmounted(() => {
    stopPolling();
    if (unlistenProgress) unlistenProgress();
    if (unlistenComplete) unlistenComplete();
    if (unlistenError) unlistenError();
  });

  return {
    transfers,
    transferList,
    activeTransfers,
    activeCount,
    loading,
    isDrawerOpen,
    refreshTransfers,
    refreshQueue: refreshTransfers,
    startPolling,
    stopPolling,
    pause,
    pauseTask: pause,
    pauseAll,
    resume,
    resumeTask: resume,
    resumeAll,
    cancel,
    cancelTask: cancel,
    retry,
    retryTask: retry,
    setPriority,
    clearCompleted,
    addUploadTask,
    addDownloadTask,
  };
}
