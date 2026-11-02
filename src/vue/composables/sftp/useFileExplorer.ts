// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { ref, computed } from "vue";
import type { FileEntry } from "../../types/sftp";

export type SortField = "name" | "size" | "modified" | "type";

export interface UseFileExplorerOptions {
  fetcher?: (path: string) => Promise<FileEntry[]>;
  initialPath?: string;
}

export function useFileExplorer(options?: UseFileExplorerOptions) {
  const currentPath = ref<string>(options?.initialPath || "/");
  const entries = ref<FileEntry[]>([]);
  const loading = ref<boolean>(false);
  const error = ref<string | null>(null);

  const history = ref<string[]>(options?.initialPath ? [options.initialPath] : ["/"]);
  const historyIndex = ref<number>(0);

  const showHidden = ref<boolean>(false);
  const searchQuery = ref<string>("");
  const sortField = ref<SortField>("name");
  const sortAsc = ref<boolean>(true);

  const selectedPaths = ref<Set<string>>(new Set());

  const canGoBack = computed(() => historyIndex.value > 0);
  const canGoForward = computed(
    () => historyIndex.value < history.value.length - 1,
  );

  async function loadDirectory(path: string, customFetcher?: (p: string) => Promise<FileEntry[]>) {
    const fetchFn = customFetcher || options?.fetcher;
    if (!fetchFn) return;

    loading.value = true;
    error.value = null;

    try {
      const data = await fetchFn(path);
      entries.value = data;
      currentPath.value = path;
      selectedPaths.value.clear();
    } catch (e: any) {
      error.value = e?.message || String(e);
      entries.value = [];
    } finally {
      loading.value = false;
    }
  }

  async function navigateTo(path: string, customFetcher?: (p: string) => Promise<FileEntry[]>) {
    // If navigating to new path, truncate future history
    if (historyIndex.value < history.value.length - 1) {
      history.value = history.value.slice(0, historyIndex.value + 1);
    }

    history.value.push(path);
    historyIndex.value = history.value.length - 1;
    await loadDirectory(path, customFetcher);
  }

  async function goBack(customFetcher?: (p: string) => Promise<FileEntry[]>) {
    if (!canGoBack.value) return;
    historyIndex.value--;
    const path = history.value[historyIndex.value];
    await loadDirectory(path, customFetcher);
  }

  async function goForward(customFetcher?: (p: string) => Promise<FileEntry[]>) {
    if (!canGoForward.value) return;
    historyIndex.value++;
    const path = history.value[historyIndex.value];
    await loadDirectory(path, customFetcher);
  }

  async function goUp(customFetcher?: (p: string) => Promise<FileEntry[]>) {
    const cur = currentPath.value.replace(/\/+$/, "");
    const parent = cur.substring(0, cur.lastIndexOf("/")) || "/";
    await navigateTo(parent, customFetcher);
  }

  function toggleSelect(path: string, multi = false) {
    if (!multi) {
      if (selectedPaths.value.has(path) && selectedPaths.value.size === 1) {
        selectedPaths.value.clear();
      } else {
        selectedPaths.value.clear();
        selectedPaths.value.add(path);
      }
    } else {
      if (selectedPaths.value.has(path)) {
        selectedPaths.value.delete(path);
      } else {
        selectedPaths.value.add(path);
      }
    }
  }

  function selectAll() {
    selectedPaths.value = new Set(filteredEntries.value.map((e) => e.path));
  }

  function clearSelection() {
    selectedPaths.value.clear();
  }

  const filteredEntries = computed(() => {
    let list = entries.value;

    if (!showHidden.value) {
      list = list.filter((e) => !e.name.startsWith("."));
    }

    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase();
      list = list.filter((e) => e.name.toLowerCase().includes(q));
    }

    return [...list].sort((a, b) => {
      // Folders always first
      const aIsDir = a.fileType === "directory";
      const bIsDir = b.fileType === "directory";
      if (aIsDir !== bIsDir) {
        return aIsDir ? -1 : 1;
      }

      let res = 0;
      switch (sortField.value) {
        case "name":
          res = a.name.localeCompare(b.name, undefined, { numeric: true });
          break;
        case "size":
          res = (a.size || 0) - (b.size || 0);
          break;
        case "modified":
          res =
            new Date(a.modified).getTime() - new Date(b.modified).getTime();
          break;
        case "type":
          res = (a.fileType || "").localeCompare(b.fileType || "");
          break;
      }
      return sortAsc.value ? res : -res;
    });
  });

  function setSort(field: SortField) {
    if (sortField.value === field) {
      sortAsc.value = !sortAsc.value;
    } else {
      sortField.value = field;
      sortAsc.value = true;
    }
  }

  return {
    currentPath,
    entries,
    filteredEntries,
    loading,
    error,
    showHidden,
    searchQuery,
    sortField,
    sortAsc,
    selectedPaths,
    canGoBack,
    canGoForward,
    loadDirectory,
    navigateTo,
    goBack,
    goForward,
    goUp,
    toggleSelect,
    selectAll,
    clearSelection,
    setSort,
  };
}
