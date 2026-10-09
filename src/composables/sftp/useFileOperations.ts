// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { ref } from "vue";
import {
  createSFTPDirectory,
  deleteSFTP,
  renameSFTP,
  setSFTPPermissions,
  writeSFTPFile,
  uploadSFTPFile,
  downloadSFTPFile,
} from "../../services/sftp";
import {
  createLocalDirectory,
  createLocalFile,
  removeLocalPath,
  renameLocalPath,
} from "../../services/localFs";

export function useFileOperations() {
  const operating = ref<boolean>(false);
  const opError = ref<string | null>(null);

  // --- Remote Operations ---
  async function createDirectory(sessionId: string, path: string): Promise<boolean> {
    operating.value = true;
    opError.value = null;
    try {
      await createSFTPDirectory(sessionId, path);
      return true;
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return false;
    } finally {
      operating.value = false;
    }
  }

  async function createFile(
    sessionId: string,
    path: string,
    content = "",
  ): Promise<boolean> {
    operating.value = true;
    opError.value = null;
    try {
      await writeSFTPFile(sessionId, path, content);
      return true;
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return false;
    } finally {
      operating.value = false;
    }
  }

  async function renameItem(
    sessionId: string,
    oldPath: string,
    newPath: string,
  ): Promise<boolean> {
    operating.value = true;
    opError.value = null;
    try {
      await renameSFTP(sessionId, oldPath, newPath);
      return true;
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return false;
    } finally {
      operating.value = false;
    }
  }

  async function deleteItems(
    sessionId: string,
    paths: string[],
    recursive = true,
  ): Promise<boolean> {
    operating.value = true;
    opError.value = null;
    try {
      for (const p of paths) {
        await deleteSFTP(sessionId, p, recursive);
      }
      return true;
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return false;
    } finally {
      operating.value = false;
    }
  }

  async function changePermissions(
    sessionId: string,
    path: string,
    mode: number,
  ): Promise<boolean> {
    operating.value = true;
    opError.value = null;
    try {
      await setSFTPPermissions(sessionId, path, mode);
      return true;
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return false;
    } finally {
      operating.value = false;
    }
  }

  // --- Local Operations ---
  async function createLocalDir(path: string): Promise<boolean> {
    operating.value = true;
    opError.value = null;
    try {
      await createLocalDirectory(path);
      return true;
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return false;
    } finally {
      operating.value = false;
    }
  }

  async function createLocalNewFile(path: string, content = ""): Promise<boolean> {
    operating.value = true;
    opError.value = null;
    try {
      await createLocalFile(path, content);
      return true;
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return false;
    } finally {
      operating.value = false;
    }
  }

  async function renameLocal(oldPath: string, newPath: string): Promise<boolean> {
    operating.value = true;
    opError.value = null;
    try {
      await renameLocalPath(oldPath, newPath);
      return true;
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return false;
    } finally {
      operating.value = false;
    }
  }

  async function deleteLocalItems(paths: string[]): Promise<boolean> {
    operating.value = true;
    opError.value = null;
    try {
      for (const p of paths) {
        await removeLocalPath(p);
      }
      return true;
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return false;
    } finally {
      operating.value = false;
    }
  }

  // --- Transfers ---
  async function uploadFile(
    sessionId: string,
    localPath: string,
    remotePath: string,
  ): Promise<string | null> {
    try {
      return await uploadSFTPFile(sessionId, localPath, remotePath);
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return null;
    }
  }

  async function downloadFile(
    sessionId: string,
    remotePath: string,
    localPath: string,
  ): Promise<string | null> {
    try {
      return await downloadSFTPFile(sessionId, remotePath, localPath);
    } catch (e: any) {
      opError.value = e?.message || String(e);
      return null;
    }
  }

  return {
    operating,
    opError,
    createDirectory,
    createFile,
    renameItem,
    deleteItems,
    changePermissions,
    createLocalDir,
    createLocalNewFile,
    renameLocal,
    deleteLocalItems,
    uploadFile,
    downloadFile,
  };
}
