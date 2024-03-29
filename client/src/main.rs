use chat_client::{app::Application, tui::TUI};
use log::{error, info};

#[tokio::main]
async fn main() {
    info!("Starting client");

    // Setup env_logger to write to stderr
    env_logger::init();

    // Get Name
    let mut name = String::new();
    println!("Enter your name: ");
    std::io::stdin().read_line(&mut name).unwrap();
    let name = match name.trim() {
        "" => "Anonymous",
        name => name,
    };

    // Get Group to join
    let mut group = String::new();
    println!("Which group chat do you want to join (leave empty for default value 'general'): ");
    std::io::stdin().read_line(&mut group).unwrap();
    let group = match group.trim() {
        "" => "general",
        group => group,
    };

    // Get Server Address
    let mut address = String::new();
    println!(
        "Enter server address and port (leave empty for default value 'ws://127.0.0.1:9001'): "
    );
    std::io::stdin().read_line(&mut address).unwrap();
    let address = match address.trim() {
        "" => "ws://127.0.0.1:9001",
        address => address,
    };

    // Run until the application returns false
    while Application::new(address, name, group).run().await {
        error!("Application Disconnected. Press any key to reconnect");
    }

    TUI::exit().expect("Failed to reset terminal");
}
