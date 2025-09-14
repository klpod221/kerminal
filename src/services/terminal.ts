import { invoke } from "@tauri-apps/api/core";
// import { listen } from "@tauri-apps/api/event";

export const createTerminal = async (terminalId: string) => {
  await invoke('terminal_create_local', { terminalId });
};
