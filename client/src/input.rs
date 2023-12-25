use crossterm::event::{EventStream, KeyCode, KeyEvent};
use futures_util::{FutureExt, StreamExt};
use log::error;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::Event;

pub struct EventHandler {
    channel: mpsc::UnboundedReceiver<Event>,
    task: JoinHandle<()>,
}

impl EventHandler {
    pub fn new() -> Self {
        // Create a channel to send events from the event handler to the main thread
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let mut reader = EventStream::new();

        // Spawn a task to read events from the terminal
        let task = tokio::spawn(async move {
            loop {
                let event = reader.next().fuse();
                match event.await {
                    Some(Ok(event)) => {
                        if let Some(message) = Self::handle_event(event) {
                            events_tx.send(message).unwrap();
                        }
                    }
                    Some(Err(e)) => error!("Error: {:?}\r", e),
                    None => break,
                }
            }
        });

        Self {
            channel: events_rx,
            task,
        }
    }

    /// Handle an event from the terminal
    fn handle_event(event: crossterm::event::Event) -> Option<Event> {
        match event {
            crossterm::event::Event::Key(KeyEvent { code, .. }) => Self::handle_key(code),
            crossterm::event::Event::Resize(width, height) => Some(Event::Resize { width, height }),
            _ => None,
        }
    }

    /// Handle a key event from the terminal
    fn handle_key(key: KeyCode) -> Option<Event> {
        match key {
            KeyCode::Esc => Some(Event::Quit),
            KeyCode::Char(c) => Some(Event::Input(c)),
            _ => None,
        }
    }

    /// Get the next event from the event handler
    pub async fn next(&mut self) -> Event {
        self.channel.recv().await.expect("Failed to receive event")
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
