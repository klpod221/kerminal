// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { api } from "./api";
import { homeDir } from "@tauri-apps/api/path";
import type { FileEntry } from "../types/sftp";

export interface LocalFileItem {
  name: string;
  is_file: boolean;
  is_dir: boolean;
  size: number;
  modified: number | null;
}

export async function getDefaultLocalPath(): Promise<string> {
  try {
    const home = await homeDir();
    return home || "/";
  } catch {
    return "/";
  }
}

export async function listLocalDirectory(path: string): Promise<FileEntry[]> {
  const normalizedPath = path.replace(/\/+$/, "") || "/";
  const rawEntries = await api.callRaw<LocalFileItem[]>("local_fs_read_dir", {
    path: normalizedPath,
  });

  return rawEntries.map((item) => {
    const fullPath =
      normalizedPath === "/" ? `/${item.name}` : `${normalizedPath}/${item.name}`;
    const fileType = item.is_dir ? "directory" : "file";

    return {
      name: item.name,
      path: fullPath,
      fileType,
      size: fileType === "file" ? item.size : null,
      permissions: 0o755,
      modified: item.modified
        ? new Date(item.modified * 1000).toISOString()
        : new Date().toISOString(),
      accessed: null,
      symlinkTarget: null,
      uid: null,
      gid: null,
    };
  });
}

export async function createLocalDirectory(path: string): Promise<void> {
  await api.callRaw("local_fs_mkdir", { path });
}

export async function createLocalFile(path: string, contents = ""): Promise<void> {
  await api.callRaw("local_fs_write_text_file", { path, contents });
}

export async function readLocalTextFile(path: string): Promise<string> {
  return await api.callRaw<string>("local_fs_read_text_file", { path });
}

export async function removeLocalPath(path: string): Promise<void> {
  await api.callRaw("local_fs_remove", { path });
}

export async function renameLocalPath(
  oldPath: string,
  newPath: string,
): Promise<void> {
  await api.callRaw("local_fs_rename", {
    oldPath,
    newPath,
    old_path: oldPath,
    new_path: newPath,
  });
}
