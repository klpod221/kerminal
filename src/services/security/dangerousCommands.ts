// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

export interface DangerousCommandPattern {
  id: string;
  name: string;
  pattern: RegExp;
  severity: "critical" | "high" | "medium";
  description: string;
}

export const DANGEROUS_PATTERNS: DangerousCommandPattern[] = [
  {
    id: "rm-root-all",
    name: "Recursive Force Delete Root/System",
    pattern: /(?:^|[;&|]\s*)(?:sudo\s+)?rm\s+-[a-zA-Z]*r[a-zA-Z]*f[a-zA-Z]*\s+(?:\/|\/\*|~|\*|\.\*|\.\.\/)(?:\s+.*)?$/i,
    severity: "critical",
    description: "Permanently deletes files recursively across root, home, or wildcards.",
  },
  {
    id: "mkfs-raw",
    name: "Format Disk / Filesystem",
    pattern: /(?:^|[;&|]\s*)(?:sudo\s+)?mkfs(?:\.[a-zA-Z0-9]+)?\s+\/dev\/[a-zA-Z0-9_-]+/i,
    severity: "critical",
    description: "Creates a new filesystem on a partition or disk, destroying all existing data.",
  },
  {
    id: "dd-disk-overwrite",
    name: "Raw Disk Direct Write (dd)",
    pattern: /(?:^|[;&|]\s*)(?:sudo\s+)?dd\s+.*of=\/dev\/(?:sd[a-z]|nvme[0-9]n[0-9]|hd[a-z]|vd[a-z]|loop[0-9]|null|zero)/i,
    severity: "critical",
    description: "Writes raw stream directly to storage device, corrupting partition tables and data.",
  },
  {
    id: "shutdown-reboot",
    name: "System Shutdown / Reboot",
    pattern: /(?:^|[;&|]\s*)(?:sudo\s+)?(?:shutdown(?:\s+-[a-zA-Z0-9]+)?|\breboot\b|\bpoweroff\b|\binit\s+0\b|\binit\s+6\b)/i,
    severity: "high",
    description: "Halts, powers off, or reboots the operating system immediately.",
  },
  {
    id: "chmod-777-root",
    name: "Full Permissive Permissions (chmod 777)",
    pattern: /(?:^|[;&|]\s*)(?:sudo\s+)?chmod\s+-[a-zA-Z]*R[a-zA-Z]*\s+777\s+(?:\/|~|\/etc|\/usr|\/var|\/bin|\/sbin)/i,
    severity: "high",
    description: "Gives unrestricted read/write/execute permissions to all users across system directories.",
  },
  {
    id: "fork-bomb",
    name: "Shell Fork Bomb",
    pattern: /:\(\)\s*\{\s*:\|:&\s*\};:/i,
    severity: "critical",
    description: "Spawns infinite recursive background processes, exhausting OS process table (DoS).",
  },
  {
    id: "kubectl-delete-all",
    name: "Kubernetes Mass Delete",
    pattern: /(?:^|[;&|]\s*)kubectl\s+delete\s+(?:all|namespace|namespaces|ns|node|nodes)\b/i,
    severity: "high",
    description: "Destroys all Kubernetes resources, entire namespaces, or cluster nodes.",
  },
  {
    id: "drop-database",
    name: "Database Drop Table/Database",
    pattern: /(?:^|[;&|]\s*)(?:drop\s+database|drop\s+table\s+.*cascade|truncate\s+table)\b/i,
    severity: "high",
    description: "Irrevocably deletes entire database schema or table data.",
  },
  {
    id: "git-reset-hard",
    name: "Git Hard Reset Without Stash",
    pattern: /(?:^|[;&|]\s*)git\s+reset\s+--hard\b/i,
    severity: "medium",
    description: "Discards all uncommitted changes and resets branch head, losing unstaged work.",
  },
];

export function detectDangerousCommand(
  rawCommandLine: string,
): DangerousCommandPattern | null {
  if (!rawCommandLine) return null;
  const trimmed = rawCommandLine.trim();
  if (trimmed.length < 3) return null;

  for (const item of DANGEROUS_PATTERNS) {
    if (item.pattern.test(trimmed)) {
      return item;
    }
  }

  return null;
}
