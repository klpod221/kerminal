// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import type { TerminalRendererPerformanceTelemetry } from "./RendererPerformanceTelemetry";
import {
  discardForegroundRenderSettle,
  writeForegroundTerminalChunk,
  type ForegroundTerminalOutputTarget,
} from "./ForegroundRenderSettle";
import {
  CURSOR_SHOW,
  removeTransientCursorShowSequences,
  TUI_SYNCHRONIZED_FLUSH_MAX_CHARS,
  TUI_SYNCHRONIZED_FRAME_COALESCE_MS,
  TUI_SYNCHRONIZED_FRAME_HOLD_MS,
} from "./TuiCursorProtection";

type TerminalOutputSink = ForegroundTerminalOutputTarget;

export type TerminalOutputCadence = "focused" | "visible" | "hidden";
export type TerminalOutputCallbackMode = "auto" | "required" | "unsupported";

export interface TerminalOutputScheduler {
  cancel(handle: number): void;
  request(callback: () => void, delayMs?: number): number;
}

export interface TerminalOutputWriterOptions {
  adaptive?: boolean;
  callbackMode?: TerminalOutputCallbackMode;
  cadence?: TerminalOutputCadence;
  cadenceDelaysMs?: Partial<Record<TerminalOutputCadence, number>>;
  initialCharsPerFlush?: number;
  maxCharsPerFlush?: number;
  minCharsPerFlush?: number;
  now?: () => number;
  onWriteError?: (error: unknown, data: string) => void;
  scheduler?: TerminalOutputScheduler;
  slowFlushMs?: number;
  targetWriteCallbackMs?: number;
  telemetry?: TerminalRendererPerformanceTelemetry;
}

export interface TerminalOutputWriterStats {
  adaptationDecreaseCount: number;
  adaptationIncreaseCount: number;
  currentCharsPerFlush: number;
  drainCount: number;
  inFlight: boolean;
  flushCount: number;
  lastDrainMs?: number;
  lastFlushChars: number;
  lastFlushMs?: number;
  lastSlowFlushAt?: number;
  maxDrainMs: number;
  maxFlushMs: number;
  pendingBytes: number;
  pendingChars: number;
  pendingChunks: number;
  pendingHighWaterChars: number;
  slowFlushCount: number;
  splitFrameCount: number;
  targetWriteCallbackMs: number;
  totalFlushChars: number;
  writeErrorCount: number;
  writeNowCount: number;
}

export interface TerminalOutputWriter {
  dispose(): void;
  flush(): void;
  isTuiCursorProtectionActive(): boolean;
  pendingLength(): number;
  setCadence(cadence: TerminalOutputCadence): void;
  setTuiCursorProtection(active: boolean): void;
  stats(): TerminalOutputWriterStats;
  write(data: string): void;
  writeNow(data: string): void;
}

const DEFAULT_MIN_CHARS_PER_FLUSH = 4 * 1024;
const DEFAULT_INITIAL_CHARS_PER_FLUSH = 16 * 1024;
const DEFAULT_MAX_CHARS_PER_FLUSH = 64 * 1024;
const DEFAULT_TARGET_WRITE_CALLBACK_MS = 6;
const DEFAULT_SLOW_FLUSH_MS = 16;
const FRAME_FALLBACK_MS = 16;
const ADAPTATION_HYSTERESIS_SAMPLES = 2;
const HIDDEN_PRESSURE_THRESHOLD_CHARS = 256 * 1024;

const DEFAULT_CADENCE_DELAYS_MS: Record<TerminalOutputCadence, number> = {
  focused: 0,
  visible: 16,
  hidden: 100,
};

const browserFrameScheduler: TerminalOutputScheduler = {
  cancel: (handle) => globalThis.clearTimeout(handle),
  request: (cb, delayMs = 0) => {
    if (delayMs <= 0 && typeof globalThis.requestAnimationFrame === "function") {
      const id = globalThis.requestAnimationFrame(() => cb());
      return id as unknown as number; // Hack to store both timeout and raf ids
    }
    return globalThis.setTimeout(cb, delayMs) as unknown as number;
  },
};

function clampFinite(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.max(min, Math.min(max, value));
}

function utf8ByteLength(str: string): number {
  let len = 0;
  for (let i = 0; i < str.length; i++) {
    const code = str.charCodeAt(i);
    if (code <= 0x7f) len += 1;
    else if (code <= 0x7ff) len += 2;
    else if (code >= 0xd800 && code <= 0xdfff) {
      len += 4;
      i++;
    } else len += 3;
  }
  return len;
}

function safeSplitIndex(str: string, targetLength: number, allowEmpty: boolean): number {
  if (targetLength <= 0 && allowEmpty) return 0;
  if (targetLength >= str.length) return str.length;
  // Don't split surrogate pairs
  let index = targetLength;
  if (index > 0 && index < str.length) {
    const code = str.charCodeAt(index - 1);
    if (code >= 0xd800 && code <= 0xdbff) {
      index -= 1;
    }
  }
  // Don't split ANSI sequences if possible
  const lastEscape = str.lastIndexOf("\x1b", index - 1);
  if (lastEscape !== -1) {
    // Check if the escape sequence is complete
    let isComplete = false;
    for (let i = lastEscape + 1; i < index; i++) {
      const c = str[i];
      if (c === undefined) break;
      if (
        (c >= "a" && c <= "z") ||
        (c >= "A" && c <= "Z") ||
        c === "~" ||
        c === "@" ||
        c === "`"
      ) {
        isComplete = true;
        break;
      }
    }
    if (!isComplete && index - lastEscape < 16) {
      index = lastEscape;
    }
  }
  return Math.max(allowEmpty ? 0 : 1, index);
}

export function createTerminalOutputWriter(
  terminal: TerminalOutputSink,
  options: TerminalOutputWriterOptions = {},
): TerminalOutputWriter {
  const scheduler = options.scheduler ?? browserFrameScheduler;
  const now = options.now ?? (() => Date.now());
  const telemetry = options.telemetry;
  const adaptive = options.adaptive ?? true;
  const callbackMode = options.callbackMode ?? "auto";
  const slowFlushMs = Math.max(0, options.slowFlushMs ?? DEFAULT_SLOW_FLUSH_MS);
  const targetWriteCallbackMs = clampFinite(
    options.targetWriteCallbackMs ?? DEFAULT_TARGET_WRITE_CALLBACK_MS,
    1,
    50,
  );
  const requestedMaxCharsPerFlush = Math.max(
    1,
    Math.floor(options.maxCharsPerFlush ?? DEFAULT_MAX_CHARS_PER_FLUSH),
  );
  const minCharsPerFlush = Math.max(
    1,
    Math.min(
      requestedMaxCharsPerFlush,
      Math.floor(
        options.minCharsPerFlush ??
          Math.min(DEFAULT_MIN_CHARS_PER_FLUSH, requestedMaxCharsPerFlush),
      ),
    ),
  );
  const maxCharsPerFlush = Math.max(minCharsPerFlush, requestedMaxCharsPerFlush);
  let currentCharsPerFlush = clampFinite(
    Math.floor(options.initialCharsPerFlush ?? DEFAULT_INITIAL_CHARS_PER_FLUSH),
    minCharsPerFlush,
    maxCharsPerFlush,
  );
  const cadenceDelaysMs = {
    ...DEFAULT_CADENCE_DELAYS_MS,
    ...options.cadenceDelaysMs,
  };
  const chunks: string[] = [];
  let adaptationDirection: "increase" | "decrease" | null = null;
  let adaptationStreak = 0;
  let cadence = options.cadence ?? "focused";
  let chunkHead = 0;
  let disposed = false;
  let drainStartedAt: number | undefined;
  let inFlight = false;
  let pendingBytes = 0;
  let pendingChars = 0;
  let scheduledHandle: number | null = null;
  let synchronizedOutputActive = false;
  let tuiCoalescedFlushPending = false;
  let tuiPostFrameCoalesceActive = false;
  let tuiCursorProtection = false;
  const flushStats: Omit<TerminalOutputWriterStats, "currentCharsPerFlush" | "inFlight" | "pendingBytes" | "pendingChars" | "pendingChunks" | "targetWriteCallbackMs"> = {
    adaptationDecreaseCount: 0,
    adaptationIncreaseCount: 0,
    drainCount: 0,
    flushCount: 0,
    lastDrainMs: undefined,
    lastFlushChars: 0,
    lastFlushMs: undefined,
    lastSlowFlushAt: undefined,
    maxDrainMs: 0,
    maxFlushMs: 0,
    pendingHighWaterChars: 0,
    slowFlushCount: 0,
    splitFrameCount: 0,
    totalFlushChars: 0,
    writeErrorCount: 0,
    writeNowCount: 0,
  };

  const callbackSupported =
    callbackMode === "required" ||
    (callbackMode === "auto" && terminal.write.length >= 2);

  const cancelScheduledFlush = () => {
    if (scheduledHandle === null) return;
    scheduler.cancel(scheduledHandle);
    // Try to cancel RAF if it was RAF, clear timeout if timeout
    if (typeof globalThis.cancelAnimationFrame === "function") {
      globalThis.cancelAnimationFrame(scheduledHandle);
    }
    scheduledHandle = null;
  };

  const pendingChunkCount = () => Math.max(0, chunks.length - chunkHead);

  const syncTelemetry = () => {
    telemetry?.setResources({
      pendingBytes,
      pendingChunks: pendingChunkCount(),
    });
  };

  const scheduleFlush = (immediate = false, delayOverrideMs?: number) => {
    if (disposed || inFlight || scheduledHandle !== null || pendingChars === 0) {
      return;
    }
    const configuredDelay = Math.max(0, delayOverrideMs ?? cadenceDelaysMs[cadence] ?? 0);
    const pressureDelay =
      cadence === "hidden" && pendingChars >= HIDDEN_PRESSURE_THRESHOLD_CHARS
        ? FRAME_FALLBACK_MS
        : configuredDelay;
    
    // In Vue/Browser, requestAnimationFrame takes priority if delay is 0
    if ((immediate || pressureDelay === 0) && typeof globalThis.requestAnimationFrame === "function") {
      scheduledHandle = globalThis.requestAnimationFrame(flushFrame) as unknown as number;
    } else {
      scheduledHandle = globalThis.setTimeout(flushFrame, immediate ? 0 : pressureDelay) as unknown as number;
    }
  };

  const schedulePendingFlush = () => {
    if (tuiCursorProtection && synchronizedOutputActive) {
      scheduleFlush(false, TUI_SYNCHRONIZED_FRAME_HOLD_MS);
      return;
    }
    if (tuiCursorProtection && tuiPostFrameCoalesceActive) {
      scheduleFlush(false, TUI_SYNCHRONIZED_FRAME_COALESCE_MS);
      return;
    }
    scheduleFlush();
  };

  const compactQueue = () => {
    if (chunkHead === 0) return;
    if (chunkHead >= chunks.length) {
      chunks.length = 0;
      chunkHead = 0;
      return;
    }
    if (chunkHead >= 1024 && chunkHead * 2 >= chunks.length) {
      chunks.splice(0, chunkHead);
      chunkHead = 0;
    }
  };

  const takeBatch = (maxChars: number) => {
    let remaining = maxChars;
    let batch = "";

    while (chunkHead < chunks.length && remaining > 0) {
      const current = chunks[chunkHead] ?? "";
      if (current.length <= remaining) {
        batch += current;
        chunkHead += 1;
        pendingChars -= current.length;
        pendingBytes -= utf8ByteLength(current);
        remaining -= current.length;
        continue;
      }

      const splitAt = safeSplitIndex(current, remaining, batch.length === 0);
      if (splitAt <= 0) break;
      const consumed = current.slice(0, splitAt);
      batch += consumed;
      chunks[chunkHead] = current.slice(splitAt);
      pendingChars -= splitAt;
      pendingBytes -= utf8ByteLength(consumed);
      if (splitAt < current.length) {
        flushStats.splitFrameCount += 1;
      }
      break;
    }

    compactQueue();
    syncTelemetry();
    return batch;
  };

  const applyAdaptation = (durationMs: number, batchChars: number) => {
    if (!adaptive) return;
    let direction: "increase" | "decrease" | null = null;
    if (durationMs > targetWriteCallbackMs * 1.25) {
      direction = "decrease";
    } else if (durationMs < targetWriteCallbackMs * 0.65 && batchChars >= currentCharsPerFlush * 0.8) {
      direction = "increase";
    }

    if (!direction) {
      adaptationDirection = null;
      adaptationStreak = 0;
      return;
    }
    if (adaptationDirection === direction) {
      adaptationStreak += 1;
    } else {
      adaptationDirection = direction;
      adaptationStreak = 1;
    }
    if (adaptationStreak < ADAPTATION_HYSTERESIS_SAMPLES) return;

    adaptationStreak = 0;
    const next =
      direction === "increase"
        ? Math.ceil(currentCharsPerFlush * 1.25)
        : Math.floor(currentCharsPerFlush * 0.75);
    const clamped = Math.max(minCharsPerFlush, Math.min(maxCharsPerFlush, next));
    if (clamped === currentCharsPerFlush) return;
    
    currentCharsPerFlush = clamped;
    if (direction === "increase") {
      flushStats.adaptationIncreaseCount += 1;
    } else {
      flushStats.adaptationDecreaseCount += 1;
    }
  };

  const finishDrainIfIdle = (completedAt: number) => {
    if (pendingChars > 0 || inFlight || drainStartedAt === undefined) return;
    const drainMs = Math.max(0, completedAt - drainStartedAt);
    flushStats.drainCount += 1;
    flushStats.lastDrainMs = drainMs;
    flushStats.maxDrainMs = Math.max(flushStats.maxDrainMs, drainMs);
    telemetry?.recordDuration("drainMs", drainMs);
    drainStartedAt = undefined;
  };

  const recordCompletedWrite = (batch: string, startedAt: number, completedAt: number) => {
    const durationMs = Math.max(0, completedAt - startedAt);
    flushStats.flushCount += 1;
    flushStats.lastFlushChars = batch.length;
    flushStats.lastFlushMs = durationMs;
    flushStats.maxFlushMs = Math.max(flushStats.maxFlushMs, durationMs);
    flushStats.totalFlushChars += batch.length;
    telemetry?.recordDuration("writeCallbackMs", durationMs);
    if (durationMs >= slowFlushMs) {
      flushStats.slowFlushCount += 1;
      flushStats.lastSlowFlushAt = completedAt;
    }
    applyAdaptation(durationMs, batch.length);
  };

  const writeBatch = (batch: string) => {
    if (!batch || disposed || inFlight) return;
    const protectedBatch = tuiCursorProtection
      ? removeTransientCursorShowSequences(batch)
      : batch;
    const settleForegroundRender =
      tuiCursorProtection &&
      (batch.includes("\x1b[?2026") || batch.includes(CURSOR_SHOW));
    
    inFlight = true;
    const startedAt = now();
    let completed = false;
    
    const complete = () => {
      if (completed) return;
      completed = true;
      const completedAt = now();
      recordCompletedWrite(protectedBatch, startedAt, completedAt);
      inFlight = false;
      finishDrainIfIdle(completedAt);
      schedulePendingFlush();
    };

    const fail = (error: unknown) => {
      if (completed) return;
      completed = true;
      inFlight = false;
      flushStats.writeErrorCount += 1;
      options.onWriteError?.(error, protectedBatch);
      finishDrainIfIdle(now());
      schedulePendingFlush();
    };

    try {
      if (callbackSupported && settleForegroundRender) {
        writeForegroundTerminalChunk(terminal, protectedBatch, {
          followupViewportRefresh: batch.includes(CURSOR_SHOW),
          forceViewportRefresh: true,
          onParsed: complete,
          onWriteFailure: fail,
        });
      } else if (callbackSupported) {
        terminal.write(protectedBatch, complete);
      } else {
        terminal.write(protectedBatch);
        complete();
      }
    } catch (error: unknown) {
      fail(error);
    }
  };

  function flushFrame() {
    scheduledHandle = null;
    if (disposed || inFlight) return;
    const batchLimit = tuiCoalescedFlushPending
      ? Math.max(currentCharsPerFlush, Math.min(pendingChars, TUI_SYNCHRONIZED_FLUSH_MAX_CHARS))
      : currentCharsPerFlush;
    tuiCoalescedFlushPending = false;
    tuiPostFrameCoalesceActive = false;
    writeBatch(takeBatch(batchLimit));
  }

  return {
    dispose() {
      if (disposed) return;
      disposed = true;
      cancelScheduledFlush();
      discardForegroundRenderSettle(terminal);
      chunks.length = 0;
      chunkHead = 0;
      pendingChars = 0;
      pendingBytes = 0;
    },
    flush() {
      if (disposed || pendingChars === 0 || inFlight) return;
      cancelScheduledFlush();
      flushFrame();
    },
    isTuiCursorProtectionActive: () => tuiCursorProtection,
    pendingLength: () => pendingChars,
    setCadence(newCadence: TerminalOutputCadence) {
      if (disposed || cadence === newCadence) return;
      cadence = newCadence;
      if (scheduledHandle !== null && cadence === "focused") {
        cancelScheduledFlush();
        scheduleFlush(true);
      } else if (scheduledHandle === null && pendingChars > 0 && !inFlight) {
        schedulePendingFlush();
      }
    },
    setTuiCursorProtection(active: boolean) {
      tuiCursorProtection = active;
      if (!active) {
        synchronizedOutputActive = false;
        tuiCoalescedFlushPending = false;
        tuiPostFrameCoalesceActive = false;
      }
    },
    stats: () => ({
      ...flushStats,
      currentCharsPerFlush,
      inFlight,
      pendingBytes,
      pendingChars,
      pendingChunks: pendingChunkCount(),
      targetWriteCallbackMs,
    }),
    write(data: string) {
      if (!data || disposed) return;
      chunks.push(data);
      pendingChars += data.length;
      pendingBytes += utf8ByteLength(data);
      if (pendingChars > flushStats.pendingHighWaterChars) {
        flushStats.pendingHighWaterChars = pendingChars;
      }
      if (drainStartedAt === undefined) {
        drainStartedAt = now();
      }
      syncTelemetry();
      schedulePendingFlush();
    },
    writeNow(data: string) {
      if (!data || disposed) return;
      if (inFlight) {
        this.write(data);
        return;
      }
      flushStats.writeNowCount += 1;
      writeBatch(data);
    },
  };
}
