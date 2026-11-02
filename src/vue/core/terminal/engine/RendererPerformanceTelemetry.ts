// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

type TerminalRendererDurationMetric =
  | "drainMs"
  | "frameGapMs"
  | "inputEchoMs"
  | "writeCallbackMs";

type TerminalRendererCounterMetric =
  | "atlasClearCount"
  | "fullRefreshCount"
  | "rendererRebuildCount"
  | "rendererSwapCount"
  | "staleCommitRejectedCount";

interface TerminalRendererDurationSummary {
  count: number;
  max: number;
  p50: number;
  p95: number;
  p99: number;
}

interface TerminalRendererResourceSnapshot {
  activeCanvases: number;
  activeContexts: number;
  activeGpuPanes: number;
  pendingBytes: number;
  pendingChunks: number;
}

export interface TerminalRendererPerformanceSnapshot {
  counters: Record<TerminalRendererCounterMetric, number>;
  durations: Partial<
    Record<TerminalRendererDurationMetric, TerminalRendererDurationSummary>
  >;
  resources: TerminalRendererResourceSnapshot;
  sampleLimit: number;
}

export interface TerminalRendererPerformanceTelemetry {
  increment(metric: TerminalRendererCounterMetric, amount?: number): void;
  recordDuration(metric: TerminalRendererDurationMetric, valueMs: number): void;
  reset(): void;
  setResources(resources: Partial<TerminalRendererResourceSnapshot>): void;
  snapshot(): TerminalRendererPerformanceSnapshot;
}

const DEFAULT_SAMPLE_LIMIT = 256;

const COUNTER_METRICS: readonly TerminalRendererCounterMetric[] = [
  "atlasClearCount",
  "fullRefreshCount",
  "rendererRebuildCount",
  "rendererSwapCount",
  "staleCommitRejectedCount",
];

export function createTerminalRendererPerformanceTelemetry(
  sampleLimit = DEFAULT_SAMPLE_LIMIT,
): TerminalRendererPerformanceTelemetry {
  const resolvedSampleLimit = Math.max(1, Math.floor(sampleLimit));
  const counters: Record<TerminalRendererCounterMetric, number> = {
    atlasClearCount: 0,
    fullRefreshCount: 0,
    rendererRebuildCount: 0,
    rendererSwapCount: 0,
    staleCommitRejectedCount: 0,
  };
  const durations = new Map<TerminalRendererDurationMetric, number[]>();
  let resources: TerminalRendererResourceSnapshot = {
    activeCanvases: 0,
    activeContexts: 0,
    activeGpuPanes: 0,
    pendingBytes: 0,
    pendingChunks: 0,
  };

  return {
    increment(metric, amount = 1) {
      if (!Number.isFinite(amount) || amount <= 0) return;
      counters[metric] += amount;
    },
    recordDuration(metric, valueMs) {
      if (!Number.isFinite(valueMs) || valueMs < 0) return;
      const samples = durations.get(metric) ?? [];
      samples.push(valueMs);
      if (samples.length > resolvedSampleLimit) {
        samples.splice(0, samples.length - resolvedSampleLimit);
      }
      durations.set(metric, samples);
    },
    reset() {
      for (const metric of COUNTER_METRICS) {
        counters[metric] = 0;
      }
      durations.clear();
      resources = {
        activeCanvases: 0,
        activeContexts: 0,
        activeGpuPanes: 0,
        pendingBytes: 0,
        pendingChunks: 0,
      };
    },
    setResources(nextResources) {
      resources = {
        ...resources,
        ...nextResources,
      };
    },
    snapshot() {
      const durationSnapshot: TerminalRendererPerformanceSnapshot["durations"] =
        {};
      for (const [metric, samples] of durations.entries()) {
        if (samples.length > 0) {
          durationSnapshot[metric] = summarizeDurations(samples);
        }
      }
      return {
        counters: { ...counters },
        durations: durationSnapshot,
        resources: { ...resources },
        sampleLimit: resolvedSampleLimit,
      };
    },
  };
}

function summarizeDurations(
  samples: readonly number[],
): TerminalRendererDurationSummary {
  const sorted = [...samples].sort((left, right) => left - right);
  return {
    count: sorted.length,
    max: sorted[sorted.length - 1] ?? 0,
    p50: percentile(sorted, 0.5),
    p95: percentile(sorted, 0.95),
    p99: percentile(sorted, 0.99),
  };
}

function percentile(sorted: number[], p: number): number {
  if (sorted.length === 0) return 0;
  const index = (sorted.length - 1) * p;
  const lower = Math.floor(index);
  const upper = Math.ceil(index);
  const weight = index % 1;
  if (lower === upper) return sorted[lower]!;
  return (sorted[lower]! * (1 - weight)) + (sorted[upper]! * weight);
}
