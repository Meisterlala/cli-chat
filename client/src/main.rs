use std::{io::BufRead, sync::Arc};

use chat_client::tui::TUI;
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
    // Setup env_logger to write to stderr
    env_logger::init();

    let s = Arc::default();
    let mut tui = chat_client::tui::TUI::new(s);

    let tui_thread = tokio::spawn(async move { tui.run().await });

    let url = "wss://echo.websocket.events";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");

    let (mut write, read) = ws_stream.split();
    let (write_tx, mut write_rx) = mpsc::unbounded_channel();

    let write_thread: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        while let Some(msg) = write_rx.recv().await {
            write.send(msg).await?;
        }
        Ok(())
    });

    let output = tokio::spawn(recieve(read));

    tokio::select! {
        Ok(e) = output => println!("Websocket read thread exited: {:?}", e.err()),
        _ = write_thread => println!("Websocket write thread exited"),
        _ = tokio::signal::ctrl_c() => println!("Ctrl-C received"),
        _ = tui_thread => println!("TUI exited"),
    }

    info!("Exiting");
    TUI::exit().expect("Failed to reset terminal");
}

type WebsocketStream = SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>>;

async fn recieve(mut read: WebsocketStream) -> anyhow::Result<()> {
    while let Some(message) = read.next().await {
        let msg = message?;
        if msg.is_close() {
            info!("Received close message");
            break;
        }

        if msg.is_text() {
            let msg = msg.into_text().unwrap();
            info!("Received: {}", msg.trim());
        }
    }
    Ok(())
}
