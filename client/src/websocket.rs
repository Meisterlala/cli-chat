use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};

pub struct Websocket {
    pub read: tokio::sync::mpsc::UnboundedReceiver<String>,
    pub write: tokio::sync::mpsc::UnboundedSender<String>,
}

impl Websocket {
    pub fn loopback() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            read: rx,
            write: tx,
        }
    }

    pub async fn connect(url: &str) -> Result<Self> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;

        info!("WebSocket handshake has been successfully completed");

        let (tx_read, rx_read) = tokio::sync::mpsc::unbounded_channel();
        let (tx_write, mut rx_write) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            let (mut ws_write, mut ws_read) = ws_stream.split();

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

        Ok(Self {
            read: rx_read,
            write: tx_write,
        })
    }

    pub fn send(&mut self, msg: String) -> anyhow::Result<()> {
        self.write.send(msg).unwrap();
        Ok(())
    }

    pub async fn recieve(&mut self) -> anyhow::Result<String> {
        let msg = self.read.recv().await.unwrap();
        Ok(msg)
    }

    pub fn status(&self) -> Result<()> {
        Ok(())
    }
}
