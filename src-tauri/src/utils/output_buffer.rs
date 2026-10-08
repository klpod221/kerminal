use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

pub struct TerminalOutputBuffer {
    buffer: Mutex<Vec<u8>>,
    notify: Notify,
    max_size: usize,
    dropped_bytes: Mutex<usize>,
    is_eof: Mutex<bool>,
}

impl TerminalOutputBuffer {
    pub fn new(max_size: usize) -> Arc<Self> {
        Arc::new(Self {
            buffer: Mutex::new(Vec::with_capacity(65536)),
            notify: Notify::new(),
            max_size,
            dropped_bytes: Mutex::new(0),
            is_eof: Mutex::new(false),
        })
    }

    pub fn push(&self, data: &[u8]) {
        if data.is_empty() {
            *self.is_eof.lock().unwrap() = true;
        } else {
            let mut buf = self.buffer.lock().unwrap();
            if buf.len() + data.len() > self.max_size {
                let overflow = (buf.len() + data.len()) - self.max_size;
                let to_drop = std::cmp::min(buf.len(), overflow);
                buf.drain(0..to_drop);
                let mut dropped = self.dropped_bytes.lock().unwrap();
                *dropped += to_drop;
            }
            buf.extend_from_slice(data);
        }
        self.notify.notify_one();
    }

    pub async fn pop_all(&self) -> (Vec<u8>, usize) {
        loop {
            let (data, dropped, eof) = {
                let mut buf = self.buffer.lock().unwrap();
                let mut dropped_lock = self.dropped_bytes.lock().unwrap();
                let dropped_count = *dropped_lock;
                let eof = *self.is_eof.lock().unwrap();
                
                if buf.is_empty() && dropped_count == 0 && !eof {
                    (None, 0, false)
                } else {
                    *dropped_lock = 0;
                    (Some(std::mem::take(&mut *buf)), dropped_count, eof)
                }
            };

            if let Some(buf_data) = data {
                return (buf_data, dropped);
            }
            if eof {
                return (Vec::new(), dropped);
            }

            self.notify.notified().await;
        }
    }
}
