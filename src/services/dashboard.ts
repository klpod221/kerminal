import { api } from "./api";

export async function getSystemInfo() {
  const systemInfo = await api.callRaw("get_system_info");
  return systemInfo;
}
