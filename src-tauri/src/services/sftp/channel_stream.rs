use russh::Channel;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;

/// Wrapper to convert russh Channel into AsyncRead + AsyncWrite stream
/// Uses background tasks to handle channel I/O
pub struct ChannelStream {
    read_rx: mpsc::UnboundedReceiver<Result<Vec<u8>, std::io::Error>>,
    write_tx: mpsc::UnboundedSender<Vec<u8>>,
    read_buffer: Vec<u8>,
    read_offset: usize,
}

impl ChannelStream {
    pub fn new(channel: Channel<russh::client::Msg>) -> Self {
        let (read_tx, read_rx) = mpsc::unbounded_channel::<Result<Vec<u8>, std::io::Error>>();
        let (write_tx, mut write_rx) = mpsc::unbounded_channel::<Vec<u8>>();

        // Spawn background task to handle both read and write
        tokio::spawn(async move {
            let mut channel = channel;
            loop {
                tokio::select! {
                    // Handle writes
                    data_opt = write_rx.recv() => {
                        if let Some(data) = data_opt {
                            if channel.data(data.as_slice()).await.is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    // Handle reads
                    msg_opt = channel.wait() => {
                        match msg_opt {
                            Some(russh::ChannelMsg::Data { data }) => {
                                let _ = read_tx.send(Ok(data.to_vec()));
                            }
                            Some(russh::ChannelMsg::Eof) => {
                                let _ = read_tx.send(Err(std::io::Error::new(
                                    std::io::ErrorKind::UnexpectedEof,
                                    "Channel EOF"
                                )));
                                break;
                            }
                            Some(russh::ChannelMsg::Close) => {
                                break;
                            }
                            None => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Self {
            read_rx,
            write_tx,
            read_buffer: Vec::new(),
            read_offset: 0,
        }
    }
}

impl AsyncRead for ChannelStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        // First, consume from buffer if available
        if self.read_offset < self.read_buffer.len() {
            let available = &self.read_buffer[self.read_offset..];
            let to_copy = available.len().min(buf.remaining());
            buf.put_slice(&available[..to_copy]);
            self.read_offset += to_copy;

            // If buffer is fully consumed, clear it
            if self.read_offset >= self.read_buffer.len() {
                self.read_buffer.clear();
                self.read_offset = 0;
            }

            return Poll::Ready(Ok(()));
        }

        // Try to receive new data
        match self.read_rx.poll_recv(cx) {
            Poll::Ready(Some(Ok(data))) => {
                let to_copy = data.len().min(buf.remaining());
                buf.put_slice(&data[..to_copy]);

                // Store remaining data in buffer
                if to_copy < data.len() {
                    self.read_buffer = data[to_copy..].to_vec();
                    self.read_offset = 0;
                }

                Poll::Ready(Ok(()))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Err(e)),
            Poll::Ready(None) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Channel closed",
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for ChannelStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let data = buf.to_vec();
        match self.write_tx.send(data) {
            Ok(_) => Poll::Ready(Ok(buf.len())),
            Err(_) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Channel write failed",
            ))),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

// Mark as Unpin since we don't store any pinned data
impl Unpin for ChannelStream {}
