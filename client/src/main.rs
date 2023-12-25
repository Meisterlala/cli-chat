use std::{io::BufRead, sync::Arc};

use chat_client::{app::Application, tui::TUI};
use crossterm::style::Stylize;
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use log::info;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};

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

type WebsocketStream = SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>>;


