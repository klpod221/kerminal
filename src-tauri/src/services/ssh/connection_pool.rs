/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

struct PooledConnection {
    last_used: Instant,
    active: bool,
}

pub struct SSHConnectionPool {
    connections: Arc<RwLock<HashMap<String, PooledConnection>>>,
    max_idle_time: Duration,
}

impl SSHConnectionPool {
    pub fn new(max_idle_minutes: u64) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_idle_time: Duration::from_secs(max_idle_minutes * 60),
        }
    }

    pub async fn clear(&self) {
        let mut pool = self.connections.write().await;
        pool.clear();
    }

    pub async fn cleanup_idle(&self) {
        let mut pool = self.connections.write().await;
        pool.retain(|_, conn| conn.active && conn.last_used.elapsed() < self.max_idle_time);
    }

    pub async fn pool_size(&self) -> usize {
        let pool = self.connections.read().await;
        pool.len()
    }
}

impl Default for SSHConnectionPool {
    fn default() -> Self {
        Self::new(30)
    }
}
