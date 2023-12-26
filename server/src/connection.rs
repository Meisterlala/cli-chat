use futures_util::{SinkExt, StreamExt};
use log::{debug, error};
use tokio::net::TcpStream;

pub struct Connection {
    pub read: tokio::sync::mpsc::UnboundedReceiver<String>,
    pub write: tokio::sync::mpsc::UnboundedSender<String>,
    pub task: tokio::task::JoinHandle<()>,
}

impl Connection {
    pub fn new(stream: tokio_tungstenite::WebSocketStream<TcpStream>) -> Self {
        let (tx_read, rx_read) = tokio::sync::mpsc::unbounded_channel();
        let (tx_write, mut rx_write) = tokio::sync::mpsc::unbounded_channel();

        let t = tokio::spawn(async move {
            let (mut ws_write, mut ws_read) = stream.split();

            loop {
                tokio::select! {
                    Some(msg) = rx_write.recv() => {
                        debug!("Sending message: {}", msg);
                        ws_write.send(tokio_tungstenite::tungstenite::Message::Text(msg)).await.unwrap();
                    }
                    Some(msg) = ws_read.next() => {
                        match msg {
                            Ok(msg) => {
                                let msg = msg.into_text().expect("Failed to convert message to text");
                                debug!("Recieved message: {}", msg);
                                tx_read.send(msg).unwrap();
                            }
                            Err(e) => {
                                error!("Error reading from websocket: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });

        Self {
            read: rx_read,
            write: tx_write,
            task: t,
        }
    }

    pub fn loopback(stream: tokio_tungstenite::WebSocketStream<TcpStream>) -> Self {
        let task = tokio::spawn(async move {
            let (mut ws_write, mut ws_read) = stream.split();
            while let Some(Ok(msg)) = ws_read.next().await {
                if !msg.is_close() && !msg.is_pong() && !msg.is_empty() {
                    debug!("Resending: {}", msg);
                    ws_write.send(msg).await.unwrap();
                }
            }
            debug!("Loopback closed")
        });

        let (w, r) = tokio::sync::mpsc::unbounded_channel();
        Self {
            read: r,
            write: w,
            task,
        }
    }

    pub fn send(&mut self, msg: String) -> anyhow::Result<()> {
        self.write.send(msg)?;
        Ok(())
    }

    pub async fn recieve(&mut self) -> anyhow::Result<String> {
        let msg = self
            .read
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to recieve message"))?;
        Ok(msg)
    }
}
