use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures_util::FutureExt;
use log::{debug, error};

use crate::{connection::Connection, websocket};

type Sender = tokio::sync::mpsc::UnboundedSender<std::string::String>;

pub struct Application {
    pub adress: String,
    pub connections: Arc<Mutex<HashMap<String, Vec<Sender>>>>,
}

impl Application {
    pub fn new(adress: &str) -> Self {
        Self {
            adress: adress.to_string(),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&mut self) {
        let (connection_sender, mut connection_receiver) = tokio::sync::mpsc::unbounded_channel();
        //let (message_sender, message_receiver) = tokio::sync::mpsc::unbounded_channel();

        let adress = self.adress.clone();
        tokio::spawn(websocket::accept_connections(adress, connection_sender));

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    debug!("Ctrl-C recieved, shutting down");
                    break;
                }
                Some(connection) = connection_receiver.recv().fuse() => {
                    debug!("Client connected");
                    self.on_connection(connection);
                }

            };
        }
    }

    fn on_connection(&mut self, connection: Connection) {
        let mut read_channel = connection.receiver;
        let connections = self.connections.clone();

        tokio::spawn(async move {
            // await for the group to be set
            while connection.group.lock().unwrap().is_none() {
                tokio::task::yield_now().await;
            }

            // Add the connection to the group
            connections
                .lock()
                .unwrap()
                .entry(connection.group.lock().unwrap().as_ref().unwrap().clone())
                .or_default()
                .push(connection.sender);

            // Listen for messages
            loop {
                let msg = read_channel.recv().await;
                if let Some(msg) = msg {
                    debug!("Recieved message: {}", msg);
                    let mut connections = connections.lock().unwrap();

                    if let Some(group) = connection.group.lock().unwrap().as_ref() {
                        if let Some(connections) = connections.get_mut(group) {
                            for connection in connections.iter_mut() {
                                if let Err(e) = Connection::send(connection, msg.clone()) {
                                    error!("Error sending message: {}", e);
                                }
                            }
                        }
                    }
                } else {
                    error!("Connection closed");
                    break;
                }
            }
        });
    }
}
