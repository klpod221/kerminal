// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

const SYNCHRONIZED_OUTPUT_START = "\x1b[?2026h";
const SYNCHRONIZED_OUTPUT_END = "\x1b[?2026l";
export const CURSOR_SHOW = "\x1b[?25h";
const CURSOR_HIDE = "\x1b[?25l";
export const TUI_SYNCHRONIZED_FRAME_HOLD_MS = 32;
export const TUI_SYNCHRONIZED_FRAME_COALESCE_MS = 16;
export const TUI_SYNCHRONIZED_FLUSH_MAX_CHARS = 256 * 1024;

export interface SynchronizedOutputScanState {
  active: boolean;
  tail: string;
}

export function previewPendingData(
  chunks: string[],
  chunkHead: number,
  limit: number,
) {
  let data = "";
  for (let index = chunkHead; index < chunks.length; index += 1) {
    const remaining = limit - data.length;
    if (remaining <= 0) {
      break;
    }
    data += (chunks[index] ?? "").slice(0, remaining);
  }
  return data;
}

export function coalescedTuiFrameNeedsCursorRestore(data: string) {
  const synchronizedEndIndex = data.lastIndexOf(SYNCHRONIZED_OUTPUT_END);
  if (synchronizedEndIndex === -1) {
    return false;
  }
  const synchronizedFrame = data.slice(
    0,
    synchronizedEndIndex + SYNCHRONIZED_OUTPUT_END.length,
  );
  return (
    containsCursorRestore(synchronizedFrame) &&
    !containsFinalCursorPlacementBeforeSynchronizedEnd(synchronizedFrame) &&
    !containsCursorRestore(
      data.slice(synchronizedEndIndex + SYNCHRONIZED_OUTPUT_END.length),
    )
  );
}

export function scanSynchronizedOutput(
  data: string,
  state: SynchronizedOutputScanState,
): SynchronizedOutputScanState & { ended: boolean; started: boolean } {
  const window = state.tail + data;
  let active = state.active;
  let ended = false;
  let started = false;
  let offset = 0;

  while (offset < window.length) {
    const start = window.indexOf(SYNCHRONIZED_OUTPUT_START, offset);
    const end = window.indexOf(SYNCHRONIZED_OUTPUT_END, offset);
    if (start === -1 && end === -1) {
      break;
    }
    if (start !== -1 && (end === -1 || start < end)) {
      active = true;
      started = true;
      offset = start + SYNCHRONIZED_OUTPUT_START.length;
      continue;
    }
    active = false;
    ended = true;
    offset = end + SYNCHRONIZED_OUTPUT_END.length;
  }

  const tailLength = Math.max(
    SYNCHRONIZED_OUTPUT_START.length,
    SYNCHRONIZED_OUTPUT_END.length,
  ) - 1;
  return {
    active,
    ended,
    started,
    tail: window.slice(-tailLength),
  };
}

export function removeTransientCursorShowSequences(data: string): string {
  let result = "";
  let offset = 0;
  let showIndex = data.indexOf(CURSOR_SHOW);
  while (showIndex !== -1) {
    const nextHideIndex = data.indexOf(
      CURSOR_HIDE,
      showIndex + CURSOR_SHOW.length,
    );
    const nextPositionEnd = findCursorPositionSequenceEnd(
      data,
      showIndex + CURSOR_SHOW.length,
      nextHideIndex === -1 ? data.length : nextHideIndex,
    );
    if (nextHideIndex === -1) {
      if (nextPositionEnd === -1) {
        const synchronizedEndIndex = data.indexOf(
          SYNCHRONIZED_OUTPUT_END,
          showIndex + CURSOR_SHOW.length,
        );
        if (synchronizedEndIndex === -1) {
          break;
        }
        result += data.slice(offset, showIndex);
        result += data.slice(
          showIndex + CURSOR_SHOW.length,
          synchronizedEndIndex + SYNCHRONIZED_OUTPUT_END.length,
        );
        result += CURSOR_SHOW;
        offset = synchronizedEndIndex + SYNCHRONIZED_OUTPUT_END.length;
        showIndex = data.indexOf(CURSOR_SHOW, offset);
        continue;
      }
      result += data.slice(offset, showIndex);
      result += data.slice(showIndex + CURSOR_SHOW.length, nextPositionEnd);
      result += CURSOR_SHOW;
      offset = nextPositionEnd;
      showIndex = data.indexOf(CURSOR_SHOW, offset);
      continue;
    }
    result += data.slice(offset, showIndex);
    offset = showIndex + CURSOR_SHOW.length;
    showIndex = data.indexOf(CURSOR_SHOW, offset);
  }
  return offset === 0 ? data : result + data.slice(offset);
}

function containsCursorRestore(data: string) {
  const hideIndex = data.indexOf(CURSOR_HIDE);
  const showIndex = data.lastIndexOf(CURSOR_SHOW);
  return (
    hideIndex !== -1 &&
    showIndex > hideIndex &&
    findCursorPositionSequenceEnd(data, 0) !== -1
  );
}

function containsFinalCursorPlacementBeforeSynchronizedEnd(data: string) {
  const synchronizedEndIndex = data.lastIndexOf(SYNCHRONIZED_OUTPUT_END);
  if (synchronizedEndIndex === -1) {
    return false;
  }
  const lastShowIndex = data.lastIndexOf(CURSOR_SHOW, synchronizedEndIndex);
  if (lastShowIndex === -1) {
    return false;
  }
  const lastHideIndex = data.lastIndexOf(CURSOR_HIDE, synchronizedEndIndex);
  if (lastHideIndex > lastShowIndex) {
    return false;
  }
  return (
    findCursorPositionSequenceEnd(
      data,
      lastShowIndex + CURSOR_SHOW.length,
      synchronizedEndIndex,
    ) !== -1
  );
}

function findCursorPositionSequenceEnd(
  data: string,
  fromIndex: number,
  toIndex = data.length,
) {
  let offset = data.indexOf("\x1b[", fromIndex);
  while (offset !== -1 && offset < toIndex) {
    let index = offset + 2;
    while (index < toIndex) {
      const character = data[index];
      if (character === "G" || character === "H" || character === "f") {
        return index + 1;
      }
      if (
        character === undefined ||
        ((character < "0" || character > "9") && character !== ";")
      ) {
        break;
      }
      index += 1;
    }
    offset = data.indexOf("\x1b[", offset + 2);
  }
  return -1;
}
