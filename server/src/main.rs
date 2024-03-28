use chat_server::websocket::Websocket;
use futures_util::FutureExt;
use log::info;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Get Port
    let mut port = String::new();
    println!("Enter server port (leave empty for default value '9001'): ");
    std::io::stdin().read_line(&mut port).unwrap();
    let adress = match port.trim() {
        "" => "127.0.0.1:9001".to_string(),
        port => format!("127.0.0.1:{}", port),
    };

    let ws = Websocket::new(&adress);

    info!("Started");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl-C recieved");
        }
        r = ws.serve().fuse() => {info!("Websocket task ended with: {:?}", r)},
    }

    info!("Exiting");
}
