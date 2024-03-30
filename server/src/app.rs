use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures_util::FutureExt;
use log::{debug, error, info};

use crate::{connection::Connection, database, websocket};

type Sender = tokio::sync::mpsc::UnboundedSender<std::string::String>;

static DATABASE_URL: &str = "sqlite://database.db";

pub struct Application {
    pub adress: String,
    pub connections: Arc<Mutex<HashMap<String, Vec<Sender>>>>,
    pub db: Option<sqlx::SqlitePool>,
}

impl Application {
    pub fn new(adress: &str) -> Self {
        Self {
            adress: adress.to_string(),
            connections: Arc::new(Mutex::new(HashMap::new())),
            db: None,
        }
    }

    pub async fn run(&mut self) {
        // Connect to DB
        if self.db.is_none() {
            match database::establish_connection(DATABASE_URL).await {
                Ok(db) => {
                    self.db = Some(db);
                    info!("Connected to database at {}", DATABASE_URL);
                }
                Err(e) => {
                    error!("Failed to connect to database: {}", e);
                }
            }

            database::create_table(self.db.as_ref().unwrap()).await;
        }

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

        let db_connection = self.db.as_ref().cloned();

        tokio::spawn(async move {
            // await for the group to be set
            while connection.group.lock().unwrap().is_none() {
                tokio::task::yield_now().await;
            }
            let group = connection.group.lock().unwrap().as_ref().unwrap().clone();

            // Send all messages from the database
            if let Some(ref db) = db_connection {
                let messages = crate::database::get_messages(db, &group).await;
                info!("Sending {} messages from group '{}'", messages.len(), group);

                for message in messages {
                    if let Err(e) = Connection::send(&connection.sender, message.serialize()) {
                        error!("Error sending message: {}", e);
                    }
                }
            }

            // Add the connection to the list of connections
            connections
                .lock()
                .unwrap()
                .entry(group.clone())
                .or_default()
                .push(connection.sender);

            // Listen for messages
            loop {
                let msg = read_channel.recv().await;
                if let Some(msg) = msg {
                    debug!("Recieved message: {}", msg);

                    let parsed = match crate::ChatMessage::deserialize(&msg) {
                        Some(msg) => {
                            debug!("Message from {}: {}", msg.username, msg.message);
                            msg
                        }
                        None => {
                            error!("Failed to deserialize message: {}", msg);
                            continue;
                        }
                    };

                    // Save message to database
                    if let Some(ref db) = db_connection {
                        crate::database::insert_message(db, &group, &parsed).await;
                    }

                    // Send message to everyone in the group
                    let mut connections = connections.lock().unwrap();
                    if let Some(group) = connection.group.lock().unwrap().as_ref() {
                        if let Some(connections) = connections.get_mut(group) {
                            connections.retain(|c| {
                                if let Err(e) = Connection::send(c, msg.clone()) {
                                    error!("Error sending message: {}", e);
                                    false
                                } else {
                                    true
                                }
                            });
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
