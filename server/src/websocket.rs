use std::sync::{Arc, Mutex};

use log::*;
use tokio::net::TcpListener;

use crate::connection::Connection;
pub struct Websocket {
    adress: String,
    connections: Arc<Mutex<Vec<Connection>>>,
}

impl Websocket {
    pub fn new(adress: &str) -> Self {
        Self {
            adress: String::from(adress),
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn serve(&self) {
        let listener = TcpListener::bind(&self.adress)
            .await
            .unwrap_or_else(|_| panic!("Failed to bind to adress: {}", &self.adress));

        info!("Listening on: {}", &self.adress);

        let c = self.connections.clone();
        debug!("TCP listener started");
        while let Ok((stream, _)) = listener.accept().await {
            info!(
                "New TCP connection: {:?}",
                stream
                    .peer_addr()
                    .map_or_else(|_| "Unknown".to_owned(), |a| a.to_string())
            );

            let c2 = c.clone();
            tokio::spawn(async move {
                match tokio_tungstenite::accept_async(stream).await {
                    Ok(ws_stream) => {
                        info!(
                            "New WebSocket connection: {:?}",
                            ws_stream.get_ref().peer_addr()
                        );
                        c2.lock().unwrap().push(Connection::loopback(ws_stream));
                    }
                    Err(e) => {
                        error!("Error during the websocket handshake occurred: {}", e);
                    }
                }
            });
        }
        debug!("TCP listener ended");
    }
}
