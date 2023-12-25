use std::sync::{atomic, Mutex};

pub struct Model {
    pub counter: atomic::AtomicU32,
    pub text_area: Mutex<String>,
    pub messages: Vec<ChatMessage>,
}

pub struct ChatMessage {
    pub username: String,
    pub message: String,
}



impl Default for Model {
    fn default() -> Self {
        Self {
            counter: atomic::AtomicU32::new(0),
            text_area: Mutex::new(String::new()),
            messages: Vec::new(),
        }
    }
}
