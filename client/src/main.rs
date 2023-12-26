use chat_client::{app::Application, tui::TUI};
use log::info;

#[tokio::main]
async fn main() {
    info!("Starting client");

    // Setup env_logger to write to stderr
    env_logger::init();

    let app = Application::new("ws://127.0.0.1:9001");
    app.run().await.unwrap();

    info!("Exiting");
    TUI::exit().expect("Failed to reset terminal");
}
