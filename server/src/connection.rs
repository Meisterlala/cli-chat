use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use log::{debug, error};
use tokio::net::TcpStream;

pub struct Connection {
    pub sender: tokio::sync::mpsc::UnboundedSender<String>,
    pub receiver: tokio::sync::mpsc::UnboundedReceiver<String>,
    pub group: Arc<Mutex<Option<String>>>,
}

impl Connection {
    pub fn new(stream: tokio_tungstenite::WebSocketStream<TcpStream>) -> Self {
        let (tx_read, rx_read) = tokio::sync::mpsc::unbounded_channel();
        let (tx_write, mut rx_write) = tokio::sync::mpsc::unbounded_channel();

        let group = Arc::new(Mutex::new(None));

        let g_clone = group.clone();
        tokio::spawn(async move {
            let connected_to = stream.get_ref().peer_addr().unwrap().to_string();
            let (mut ws_write, mut ws_read) = stream.split();

            loop {
                tokio::select! {
                    Some(msg) = rx_write.recv() => {

                        debug!("<{}> Sending message: {}", connected_to, msg);
                        ws_write.send(tokio_tungstenite::tungstenite::Message::Text(msg)).await.unwrap();
                    }
                    Some(msg) = ws_read.next() => {
                        match msg {
                            Ok(msg) => {
                                let msg = msg.into_text().expect("Failed to convert message to text");
                                debug!("<{}> Recieved message: {}", connected_to, msg);

                                if g_clone.lock().unwrap().is_none() {
                                    *g_clone.lock().unwrap() = Some(msg.clone());
                                    debug!("<{}> Joined group: {}", connected_to, msg);
                                } else {
                                    tx_read.send(msg).unwrap();
                                }
                            }
                            Err(e) => {
                                error!("<{}> Error reading from websocket: {}", connected_to, e);
                                break;
                            }
                        }
                    }
                }
            }
        });
        Self {
            sender: tx_write,
            receiver: rx_read,
            group,
        }
    }

    pub async fn wait_for_group(&self) -> String {
        loop {
            if let Some(group) = self.group.lock().unwrap().as_ref() {
                return group.clone();
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    pub fn send(
        sender: &tokio::sync::mpsc::UnboundedSender<String>,
        msg: String,
    ) -> anyhow::Result<()> {
        sender.send(msg)?;
        Ok(())
    }

    pub async fn recieve(
        mut reciever: tokio::sync::mpsc::UnboundedReceiver<String>,
    ) -> anyhow::Result<String> {
        let msg = reciever
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to recieve message"))?;
        Ok(msg)
    }
}
