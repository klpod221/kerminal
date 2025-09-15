import { terminalManager } from "./terminalManager";
import { bytesToString } from "../utils/terminalUtils";
import type {
  CreateTerminalRequest,
  WriteTerminalRequest,
  ResizeTerminalRequest,
  TerminalData,
  TerminalTitleChanged,
  TerminalExited,
} from "../types/panel";

// Export all terminal operations through the manager
export const createTerminal = (request: CreateTerminalRequest) =>
  terminalManager.createTerminal(request);

export const writeToTerminal = (request: WriteTerminalRequest) =>
  terminalManager.writeToTerminal(request);

export const resizeTerminal = (request: ResizeTerminalRequest) =>
  terminalManager.resizeTerminal(request);

export const closeTerminal = (terminalId: string) =>
  terminalManager.closeTerminal(terminalId);

export const getTerminalInfo = (terminalId: string) =>
  terminalManager.getTerminalInfo(terminalId);

export const listTerminals = () =>
  terminalManager.listTerminals();

export const closeAllTerminals = () =>
  terminalManager.closeAllTerminals();

export const createLocalTerminal = (title?: string) =>
  terminalManager.createLocalTerminal(title);

export const createSSHTerminal = (
  host: string,
  port: number,
  username: string,
  password?: string,
  privateKeyPath?: string,
  title?: string
) => terminalManager.createSSHTerminal(host, port, username, password, privateKeyPath, title);

export const getUserHostname = () =>
  terminalManager.getUserHostname();

export const listenToTerminalOutput = (callback: (data: TerminalData) => void) =>
  terminalManager.setupOutputListener(callback);

export const listenToTerminalTitleChanges = (callback: (data: TerminalTitleChanged) => void) =>
  terminalManager.setupTitleListener(callback);

export const listenToTerminalExits = (callback: (data: TerminalExited) => void) =>
  terminalManager.setupExitListener(callback);// Re-export utility
export { bytesToString };
