/**
 * Performance optimization exports
 */

export { InputBatcher } from "./InputBatcher";
export { TerminalCache, terminalCache } from "./TerminalCache";
export type { CacheStats } from "./TerminalCache";
export { IncrementalBufferLoader } from "./IncrementalBufferLoader";
export type {
  SimpleTerminal,
  TerminalBufferChunk,
  LoadProgressCallback,
  LoadOptions
} from "./IncrementalBufferLoader";
