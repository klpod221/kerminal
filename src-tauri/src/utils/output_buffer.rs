use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalMetrics {
    pub coalesced_chunks: usize,
    pub dropped_bytes: usize,
    pub overflow_count: usize,
    pub total_bytes_received: usize,
    pub total_bytes_sent: usize,
    pub flush_count: usize,
}

pub struct TerminalOutputBuffer {
    buffer: Mutex<Vec<u8>>,
    notify: Notify,
    notify_drain: Notify,
    max_size: usize,
    unreported_dropped_bytes: Mutex<usize>,
    is_eof: Mutex<bool>,
    metrics: Mutex<TerminalMetrics>,
}

impl TerminalOutputBuffer {
    pub fn new(max_size: usize) -> Arc<Self> {
        Arc::new(Self {
            buffer: Mutex::new(Vec::with_capacity(65536)),
            notify: Notify::new(),
            notify_drain: Notify::new(),
            max_size,
            unreported_dropped_bytes: Mutex::new(0),
            is_eof: Mutex::new(false),
            metrics: Mutex::new(TerminalMetrics::default()),
        })
    }



    pub async fn push_async(&self, data: &[u8]) {
        if data.is_empty() {
            *self.is_eof.lock().unwrap() = true;
            self.notify.notify_one();
            return;
        }

        let mut offset = 0;
        while offset < data.len() {
            loop {
                let available = {
                    let buf = self.buffer.lock().unwrap();
                    if buf.len() < self.max_size {
                        self.max_size - buf.len()
                    } else {
                        0
                    }
                };

                if available > 0 {
                    let chunk_size = std::cmp::min(available, data.len() - offset);
                    {
                        let mut buf = self.buffer.lock().unwrap();
                        buf.extend_from_slice(&data[offset..offset + chunk_size]);
                        let mut metrics = self.metrics.lock().unwrap();
                        metrics.total_bytes_received += chunk_size;
                        metrics.coalesced_chunks += 1;
                    }
                    self.notify.notify_one();
                    offset += chunk_size;
                    break;
                }

                // Buffer full, wait for pop_all to drain it
                self.notify_drain.notified().await;
            }
        }
    }

    pub async fn pop_all(&self) -> (Vec<u8>, usize) {
        loop {
            let (data, dropped, eof) = {
                let mut buf = self.buffer.lock().unwrap();
                let mut unreported_lock = self.unreported_dropped_bytes.lock().unwrap();
                let dropped_count = *unreported_lock;
                let eof = *self.is_eof.lock().unwrap();
                
                if buf.is_empty() && dropped_count == 0 && !eof {
                    (None, 0, false)
                } else {
                    *unreported_lock = 0;
                    
                    let mut metrics = self.metrics.lock().unwrap();
                    metrics.flush_count += 1;
                    metrics.total_bytes_sent += buf.len();

                    (Some(std::mem::take(&mut *buf)), dropped_count, eof)
                }
            };

            if let Some(buf_data) = data {
                self.notify_drain.notify_waiters();
                return (buf_data, dropped);
            }
            if eof {
                return (Vec::new(), dropped);
            }

            self.notify.notified().await;
        }
    }

    pub fn get_metrics(&self) -> TerminalMetrics {
        self.metrics.lock().unwrap().clone()
    }
}
