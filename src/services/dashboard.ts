import { invoke } from "@tauri-apps/api/core";

export async function getSystemInfo() {
  const systemInfo = await invoke("get_system_info");
  return systemInfo;
}
