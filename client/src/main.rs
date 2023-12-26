use chat_client::{app::Application, tui::TUI};
use log::info;

#[tokio::main]
async fn main() {
    info!("Starting client");

    // Setup env_logger to write to stderr
    env_logger::init();

    let app = Application::new("wss://echo.websocket.events");
    app.run().await.unwrap();

    info!("Exiting");
    TUI::exit().expect("Failed to reset terminal");
}
