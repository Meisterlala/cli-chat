use std::net::TcpListener;

use chat_server::websocket::Websocket;
use futures_util::FutureExt;
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();

    let ws = Websocket::new("127.0.0.1:9001");

    info!("Strted");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {info!("Ctrl-C recieved")},
        r = ws.serve().fuse() => {info!("Websocket task ended with: {:?}", r)},
    }
    info!("Exiting");
}
