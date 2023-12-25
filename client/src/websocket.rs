use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::info;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub struct Websocket {
    pub url: String,
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Websocket {
    pub async fn new(url: &str) -> Self {
        let (ws_stream, response) = tokio_tungstenite::connect_async(url)
            .await
            .unwrap_or_else(|_| panic!("Failed to connect to websocket: {}", url));

        info!("WebSocket handshake has been successfully completed");

        Self {
            url: url.to_string(),
            socket: ws_stream,
        }
    }

    pub async fn send(&mut self, msg: String) -> anyhow::Result<()> {
        self.socket
            .send(tokio_tungstenite::tungstenite::Message::Text(msg))
            .await?;
        Ok(())
    }

    pub async fn recieve(&mut self) -> anyhow::Result<String> {
        let msg = self.socket.next().await;
        let msg = msg.expect("Failed to recieve message");
        let msg = msg.expect("Failed to recieve message");
        let msg = msg.into_text().expect("Failed to convert message to text");
        Ok(msg)
    }

    async fn reconnect(&mut self) -> anyhow::Result<()> {
        let (ws_stream, response) = tokio_tungstenite::connect_async(&self.url)
            .await
            .expect(&format!("Failed to connect to websocket: {}", self.url));

        info!("WebSocket handshake has been successfully completed");

        self.socket = ws_stream;

        Ok(())
    }

    pub fn status(&self) -> Result<()> {
        Ok(())
    }
}
