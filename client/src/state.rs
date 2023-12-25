use std::sync::{atomic, Mutex};

use crate::Event;

pub struct State {
    pub counter: atomic::AtomicU32,
    pub text_area: Mutex<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            counter: atomic::AtomicU32::new(0),
            text_area: Mutex::new(String::new()),
        }
    }

    pub fn update(&self, event: &Event) {
        match event {
            Event::Input(c) => match c {
                'c' => {
                    self.counter
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
                _ => {
                    self.text_area.lock().unwrap().push(*c);
                }
            },
            Event::Refresh => {}
            Event::Quit => {}
            Event::Resize { width, height } => {}
        };
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
