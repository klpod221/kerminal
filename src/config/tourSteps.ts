// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import type { TourStep } from "../types/tour";
import { useViewStateStore } from "../stores/viewState";

const waitFrame = () => new Promise((resolve) => setTimeout(resolve, 80));

/**
 * Tour steps configuration for Kerminal
 * Covers workspace features, security, broadcast, and system navigation
 */
export const TOUR_STEPS: TourStep[] = [
  {
    id: "welcome",
    target: ".dashboard-container",
    title: "Welcome to Kerminal! 🎉",
    description:
      "Kerminal is a modern terminal emulator, SSH manager, and DevOps workspace. Let's take a comprehensive tour of everything it can do!",
    position: "center",
    highlight: false,
  },
  {
    id: "dashboard",
    target: '[data-tour="dashboard-btn"]',
    title: "System Dashboard 📊",
    description:
      "Real-time hardware monitoring: CPU, Memory, Disk, and Network traffic at a glance.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
    beforeShow: async () => {
      useViewStateStore().setActiveView("dashboard");
      await waitFrame();
    },
  },
  {
    id: "workspace",
    target: '[data-tour="workspace-btn"]',
    title: "Terminal Workspace 💻",
    description:
      "Your central terminal powerhouse! Multi-tab sessions, split panes, low-latency GPU rendering, and advanced keyboard navigation.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
    beforeShow: async () => {
      useViewStateStore().setActiveView("workspace");
      await waitFrame();
    },
  },
  {
    id: "split-panels",
    target: '[data-tour="split-vertical-btn"]',
    title: "Split Terminal Panes 🪟",
    description:
      "Divide your view vertically or horizontally (Ctrl+K / Ctrl+L). Drag and drop tabs directly between panes to organize your ideal layout.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
    beforeShow: async () => {
      useViewStateStore().setActiveView("workspace");
      await waitFrame();
    },
  },
  {
    id: "broadcast-input",
    target: '[data-tour="broadcast-btn"]',
    title: "Broadcast Input 📡",
    description:
      "Synchronize your keystrokes across all open terminal panes simultaneously with Ctrl+Shift+B! Execute identical commands across a fleet of servers with zero effort.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
    beforeShow: async () => {
      useViewStateStore().setActiveView("workspace");
      await waitFrame();
    },
  },
  {
    id: "security-protection",
    target: '[data-tour="security-toggle-btn"]',
    title: "Dangerous Command Protection 🛡️",
    description:
      "Smart security shield! Automatically intercepts destructive commands (rm -rf /, mkfs, dd, fork bomb) and prompts for confirmation before execution. Easily toggle on or off anytime.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
    beforeShow: async () => {
      useViewStateStore().setActiveView("workspace");
      await waitFrame();
    },
  },
  {
    id: "sftp",
    target: '[data-tour="sftp-btn"]',
    title: "Dual-Pane SFTP Browser 📂",
    description:
      "Seamless file management over SSH. Browse local and remote file systems side-by-side, drag & drop to transfer files, and edit remote files directly.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
    beforeShow: async () => {
      useViewStateStore().setActiveView("sftp");
      await waitFrame();
    },
  },
  {
    id: "ssh-profiles",
    target: '[data-tour="ssh-profiles-btn"]',
    title: "SSH Profiles & Jump Hosts 🔑",
    description:
      "Store and manage remote connections securely. Supports SSH groups, Jump Hosts (bastions), custom ports, and encrypted credentials.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
    beforeShow: async () => {
      useViewStateStore().setActiveView("dashboard");
      await waitFrame();
    },
  },
  {
    id: "terminal-profiles",
    target: '[data-tour="terminal-profiles-btn"]',
    title: "Terminal Profiles",
    description:
      "Create custom terminal configurations: shell type, fonts, colors, and environment variables for different workflows.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "saved-commands",
    target: '[data-tour="saved-commands-btn"]',
    title: "Saved Commands",
    description:
      "Save frequently used commands for quick access. Organize them into folders and execute with a single click.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "recordings",
    target: '[data-tour="recordings-btn"]',
    title: "Session Recordings",
    description:
      "Record your terminal sessions for review or sharing. Supports Asciinema format for playback and export.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "tunnels",
    target: '[data-tour="tunnels-btn"]',
    title: "SSH Tunnel Manager",
    description:
      "Create and manage SSH tunnels (port forwarding). Supports Local, Remote, and Dynamic (SOCKS) tunnels.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "ssh-keys",
    target: '[data-tour="ssh-keys-btn"]',
    title: "SSH Key Manager",
    description:
      "Manage your SSH keys. Generate new keys, import existing ones, and associate them with SSH profiles.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "sync",
    target: '[data-tour="sync-btn"]',
    title: "Sync Manager",
    description:
      "Sync your data (profiles, commands, settings) across devices via cloud or file-based synchronization.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "theme",
    target: '[data-tour="theme-btn"]',
    title: "Terminal Theme",
    description:
      "Customize your terminal appearance: colors, fonts, cursor style. Choose from many built-in themes or create your own.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "shortcuts",
    target: '[data-tour="shortcuts-btn"]',
    title: "Keyboard Shortcuts",
    description:
      "View and customize keyboard shortcuts. Pro tip: Press Ctrl+Shift+P to open the Command Palette!",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "backup",
    target: '[data-tour="backup-btn"]',
    title: "Backup & Restore",
    description:
      "Backup all your application data and restore when needed. Keep your configurations safe and portable.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "master-password",
    target: '[data-tour="master-password-btn"]',
    title: "Master Password",
    description:
      "Manage your master password that protects all sensitive data. Strong encryption keeps your credentials secure.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "complete",
    target: ".dashboard-container",
    title: "You're All Set! 🚀",
    description:
      "Thank you for choosing Kerminal! We hope it empowers your workflow and makes your development journey smoother. Happy coding, and may your connections always be stable! 💻✨",
    position: "center",
    highlight: false,
    beforeShow: async () => {
      useViewStateStore().setActiveView("workspace");
      await waitFrame();
    },
  },
];

/**
 * Get total number of tour steps
 */
export const getTourStepsCount = (): number => TOUR_STEPS.length;

/**
 * Get tour step by ID
 */
export const getTourStepById = (id: string): TourStep | undefined => {
  return TOUR_STEPS.find((step) => step.id === id);
};

/**
 * Get tour step by index
 */
export const getTourStepByIndex = (index: number): TourStep | undefined => {
  return TOUR_STEPS[index];
};
