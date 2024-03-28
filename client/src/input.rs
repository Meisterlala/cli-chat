use crossterm::event::{EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures_util::{FutureExt, StreamExt};
use log::{debug, error};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::Event;

pub struct EventHandler {
    channel: mpsc::UnboundedReceiver<Event>,
    _task: JoinHandle<()>,
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
                            if events_tx.send(message).is_ok() {}
                        }
                    }
                    Some(Err(e)) => error!("Error: {:?}\r", e),
                    None => break,
                }
            }
        });

        Self {
            channel: events_rx,
            _task: task,
        }
    }

    /// Handle an event from the terminal
    fn handle_event(event: crossterm::event::Event) -> Option<Event> {
        use crossterm::event::Event as CEvent;

        match event {
            CEvent::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => Some(Event::Quit),
            CEvent::Resize(width, height) => Some(Event::Resize { width, height }),
            CEvent::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) => Self::handle_key(code),
            CEvent::Key(KeyEvent {
                kind: KeyEventKind::Release,
                ..
            }) => None,
            e => {
                debug!("Unhandled event: {:?}", e);
                None
            }
        }
    }

    /// Handle a key event from the terminal
    fn handle_key(key: KeyCode) -> Option<Event> {
        match key {
            KeyCode::Esc => Some(Event::Quit),
            KeyCode::Enter => Some(Event::Send),
            KeyCode::Backspace => Some(Event::Backspace),
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
