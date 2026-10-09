// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Throttle interval for emitting progress events to frontend (150ms)
pub const DEFAULT_PROGRESS_INTERVAL_MS: u64 = 150;

/// Snapshot of transfer progress metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSnapshot {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: Option<u64>,
}

/// Throttled progress tracker to prevent frontend IPC saturation
pub struct ProgressTracker {
    total_bytes: u64,
    transferred: Arc<AtomicU64>,
    start_time: Instant,
    last_emit_time: Mutex<Instant>,
    last_emit_bytes: AtomicU64,
    interval: Duration,
}

impl ProgressTracker {
    /// Create new progress tracker
    pub fn new(total_bytes: u64, initial_offset: u64) -> Self {
        let now = Instant::now();
        Self {
            total_bytes,
            transferred: Arc::new(AtomicU64::new(initial_offset)),
            start_time: now,
            last_emit_time: Mutex::new(now),
            last_emit_bytes: AtomicU64::new(initial_offset),
            interval: Duration::from_millis(DEFAULT_PROGRESS_INTERVAL_MS),
        }
    }

    /// Add transferred bytes
    pub fn add_bytes(&self, count: u64) -> u64 {
        self.transferred.fetch_add(count, Ordering::Relaxed) + count
    }

    /// Get current transferred bytes
    pub fn current_bytes(&self) -> u64 {
        self.transferred.load(Ordering::Relaxed)
    }

    /// Check if enough time has elapsed to emit a progress update.
    /// If yes, returns a fresh `ProgressSnapshot` and updates internal timer.
    pub fn check_emit(&self, force: bool) -> Option<ProgressSnapshot> {
        let mut last_time = match self.last_emit_time.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let now = Instant::now();
        let elapsed = now.duration_since(*last_time);

        if !force && elapsed < self.interval {
            return None;
        }

        let curr = self.transferred.load(Ordering::Relaxed);
        let prev_bytes = self.last_emit_bytes.swap(curr, Ordering::Relaxed);
        *last_time = now;

        let total = self.total_bytes;
        let percentage = if total > 0 {
            ((curr as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
        } else {
            100.0
        };

        // Windowed speed based on interval
        let delta_bytes = curr.saturating_sub(prev_bytes);
        let speed = if elapsed.as_secs_f64() > 0.0 {
            (delta_bytes as f64 / elapsed.as_secs_f64()) as u64
        } else {
            let total_elapsed = now.duration_since(self.start_time).as_secs_f64();
            if total_elapsed > 0.0 {
                (curr as f64 / total_elapsed) as u64
            } else {
                0
            }
        };

        let eta_seconds = if speed > 0 && curr < total {
            Some((total - curr) / speed)
        } else {
            None
        };

        Some(ProgressSnapshot {
            bytes_transferred: curr,
            total_bytes: total,
            percentage,
            speed_bytes_per_sec: speed,
            eta_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_add_bytes() {
        let tracker = ProgressTracker::new(1000, 0);
        assert_eq!(tracker.current_bytes(), 0);

        tracker.add_bytes(250);
        assert_eq!(tracker.current_bytes(), 250);

        let snap = tracker.check_emit(true).expect("force snapshot");
        assert_eq!(snap.bytes_transferred, 250);
        assert_eq!(snap.total_bytes, 1000);
        assert!((snap.percentage - 25.0).abs() < f64::EPSILON);
    }
}
