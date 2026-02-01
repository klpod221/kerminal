import type { TourStep } from "../types/tour";

/**
 * Tour steps configuration for Kerminal
 * Covers all 14 main features of the application
 */
export const TOUR_STEPS: TourStep[] = [
  {
    id: "welcome",
    target: ".dashboard-container",
    title: "Welcome to Kerminal! 🎉",
    description:
      "Kerminal is a modern terminal emulator and SSH manager. Let us give you a quick tour of all the features available.",
    position: "center",
    highlight: false,
  },
  {
    id: "dashboard",
    target: '[data-tour="dashboard-btn"]',
    title: "Dashboard",
    description:
      "This is your Dashboard - displaying real-time system information including CPU, Memory, Disk, and Network statistics.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "workspace",
    target: '[data-tour="workspace-btn"]',
    title: "Workspace",
    description:
      "Your main workspace with powerful terminal capabilities. Open multiple tabs, split panels vertically or horizontally, and work efficiently.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "sftp",
    target: '[data-tour="sftp-btn"]',
    title: "SFTP Browser",
    description:
      "Manage files over SFTP connections. Drag and drop to upload/download files, connect to any SSH server seamlessly.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
  },
  {
    id: "ssh-profiles",
    target: '[data-tour="ssh-profiles-btn"]',
    title: "SSH Profiles",
    description:
      "Store and manage your SSH connections securely. All credentials are encrypted with your master password.",
    position: "bottom",
    highlight: true,
    spotlightPadding: 4,
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
