pub struct Model {
    pub url: String,
    pub username: String,
    pub text_area: String,
    pub messages: Vec<ChatMessage>,
}

pub struct ChatMessage {
    pub username: String,
    pub message: String,
}

impl ChatMessage {
    pub fn serialize(&self) -> String {
        format!("{}: {}", self.username, self.message)
    }

    // Security issue: a username can contain ": ", which would break the deserialization
    pub fn deserialize(s: &str) -> Option<Self> {
        let mut parts = s.splitn(2, ": ");
        let username = parts.next()?.to_string();
        let message = parts.next()?.to_string();
        Some(Self { username, message })
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            username: String::new(),
            url: String::new(),
            text_area: String::new(),
            messages: vec![],
        }
    }
}
