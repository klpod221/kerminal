import type { Terminal } from "@xterm/xterm";

type NavigatorWithUAData = Navigator & {
  userAgentData?: {
    platform?: string;
  };
};

const getPlatformHint = (): string => {
  if (typeof navigator === "undefined") return "";

  const nav = navigator as NavigatorWithUAData;

  return nav.userAgentData?.platform || navigator.userAgent || "";
};

export const isLinuxPlatform = (): boolean =>
  getPlatformHint().toLowerCase().includes("linux");

export const getDefaultUseWebGLRenderer = (): boolean => !isLinuxPlatform();

export const loadWebGLRenderer = async (
  term: Terminal,
  enabled: boolean,
): Promise<void> => {
  if (!enabled) return;

  try {
    const { WebglAddon } = await import("@xterm/addon-webgl");
    const webglAddon = new WebglAddon();
    term.loadAddon(webglAddon);
  } catch (error) {
    console.warn("WebGL renderer failed, falling back", error);
  }
};