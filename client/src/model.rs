use std::sync::{atomic, Mutex};

use tokio_tungstenite::tungstenite::Message;

pub struct Model {
    pub counter: u32,
    pub text_area: String,
    pub messages: Vec<ChatMessage>,
}

pub struct ChatMessage {
    pub username: String,
    pub message: String,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            counter: 0,
            text_area: String::new(),
            messages: vec![
                ChatMessage {
                    username: "test".to_string(),
                    message: "wow you are really cool".to_string(),
                },
                ChatMessage {
                    username: "test2".to_string(),
                    message: "Thank you!".to_string(),
                },
                ChatMessage {
                    username: "test".to_string(),
                    message: "How was your day".to_string(),
                },
                ChatMessage {
                    username: "test2".to_string(),
                    message: "It was good".to_string(),
                },
            ],
        }
    }
}
