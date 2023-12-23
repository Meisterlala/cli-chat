use std::io::BufRead;

use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};

mod tui;

#[tokio::main]
async fn main() {
    // tui::init().expect("Failed to draw TUI");

    let url = "wss://echo.websocket.events";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, read) = ws_stream.split();
    let (write_tx, mut write_rx) = mpsc::unbounded_channel();

    let write_thread: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        while let Some(msg) = write_rx.recv().await {
            write.send(msg).await?;
        }
        Ok(())
    });

    let input = tokio::spawn(async move {
        let lines = std::io::BufReader::new(std::io::stdin()).lines();
        for line in lines {
            println!("Sending: {}", line.as_ref().unwrap());
            let line = line.expect("Failed to read line");
            write_tx.send(Message::text(line.trim())).unwrap();
        }
        println!("Terminal got closed, exiting...");
    });

    let output = tokio::spawn(recieve(read));

    tokio::select! {
        Ok(e) = output => println!("Websocket read thread exited: {:?}", e.err()),
        _ = write_thread => println!("Websocket write thread exited"),
        _ = input => println!("stdin thread exited"),
        _ = tokio::signal::ctrl_c() => println!("Ctrl-C received"),
    }
    println!("Exiting...");
    tokio::io::stdout().flush().await.unwrap();
    println!("Exited");
}

type WebsocketStream = SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>>;

async fn recieve(mut read: WebsocketStream) -> anyhow::Result<()> {
    while let Some(message) = read.next().await {
        let msg = message?;
        if msg.is_close() {
            println!("Received close message");
            break;
        }

        if msg.is_text() {
            let msg = msg.into_text().unwrap();
            println!("Received: {}", msg.trim());
        }
    }
    Ok(())
}
