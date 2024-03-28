use std::sync::{Arc, Mutex};

use futures_util::{select, FutureExt};
use log::debug;

use crate::{connection::Connection, websocket};

pub struct Application {
    pub adress: String,
    pub connections: Arc<Mutex<Vec<Connection>>>,
}

impl Application {
    pub fn new(adress: &str) -> Self {
        Self {
            adress: adress.to_string(),
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn run(&mut self) {
        let (connection_sender, mut connection_receiver) = tokio::sync::mpsc::unbounded_channel();

        let adress = self.adress.clone();
        tokio::spawn(websocket::accept_connections(adress, connection_sender));

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    debug!("Ctrl-C recieved, shutting down");
                    break;
                }
                Some(connection) = connection_receiver.recv().fuse() => {
                    debug!("Connection recieved");
                    self.connections.lock().unwrap().push(connection);
                }
            };
        }
    }
}
