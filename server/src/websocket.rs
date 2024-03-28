use std::sync::{Arc, Mutex};

use log::*;
use tokio::net::TcpListener;

use crate::connection::Connection;

pub async fn accept_connections(
    adress: String,
    connection_stream: tokio::sync::mpsc::UnboundedSender<Connection>,
) {
    let listener = TcpListener::bind(&adress)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to adress: {}", &adress));

    info!("Listening on: {}", adress);

    debug!("TCP listener started");
    while let Ok((stream, _)) = listener.accept().await {
        info!(
            "New TCP connection: {:?}",
            stream
                .peer_addr()
                .map_or_else(|_| "Unknown".to_owned(), |a| a.to_string())
        );

        let c_clone = connection_stream.clone();
        tokio::spawn(async move {
            match tokio_tungstenite::accept_async(stream).await {
                Ok(ws_stream) => {
                    info!(
                        "New WebSocket connection: {:?}",
                        ws_stream.get_ref().peer_addr()
                    );
                    c_clone.send(Connection::new(ws_stream)).unwrap();
                }
                Err(e) => {
                    error!("Error during the websocket handshake occurred: {}", e);
                }
            }
        });
    }
    debug!("TCP listener ended");
}
