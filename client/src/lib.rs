pub mod input;
pub mod state;
pub mod tui;

pub enum Event {
    Input(char),
    Refresh,
    Quit,
    Resize { width: u16, height: u16 },
}
