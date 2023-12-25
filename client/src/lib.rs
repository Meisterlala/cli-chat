pub mod app;
pub mod input;
pub mod model;
pub mod tui;
pub mod websocket;

pub enum Event {
    Input(char),
    Refresh,
    Quit,
    Resize { width: u16, height: u16 },
    Send,
    Backspace,
}
