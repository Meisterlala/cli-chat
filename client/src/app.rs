use futures_util::{select, FutureExt};
use log::{error, info};

use crate::{
    input::EventHandler,
    model::{ChatMessage, Model},
    tui::TUI,
    websocket::Websocket,
    Event,
};

pub struct Application {
    pub url: String,
    pub user_name: String,
    pub group: String,
    pub input: EventHandler,
    pub tui: TUI,
    pub model: Model,
    pub ws: Websocket,
}

impl Application {
    pub fn new(ws_url: &str, user_name: &str, group: &str) -> Self {
        Self {
            url: ws_url.to_string(),
            user_name: user_name.to_string(),
            group: group.to_string(),
            tui: TUI::new(),
            input: EventHandler::new(),
            model: Model {
                url: ws_url.to_string(),
                username: user_name.to_string(),
                group: group.to_string(),
                ..Default::default()
            },
            ws: Websocket::loopback(),
        }
    }

    pub async fn run(mut self) -> bool {
        info!("Starting Application");

        // Wait for server start
        self.ws = Self::wait_for_websocket(&self.url).await;

        TUI::initialize_panic_handler();
        self.tui.enter().unwrap();

        // Send group to server
        if let Err(e) = self.ws.send(self.group.clone()) {
            error!("Faled to inform server which group you are joining: {}", e);
            return false;
        }

        let render_thread = tokio::spawn(async move {
            loop {
                self.tui.render(&self.model).await.ok().or_else(|| {
                    info!("Failed to render");
                    None
                });

                let event = select! {
                    msg = self.input.next().fuse() => {
                        msg
                    }
                    msg = self.ws.recieve().fuse() => {
                        // Cant recive message, restart
                        if msg.is_err() {
                            error!("Failed to recive message, restarting");
                            Event::Restart
                        // Deserialize message
                        } else if let Some(msg) = ChatMessage::deserialize(&msg.unwrap()) {
                            Event::ReciveMessage(msg)
                        } else {
                            continue;
                        }
                    }
                };

                match event {
                    Event::Quit | Event::Restart => {
                        TUI::exit().unwrap();

                        return event;
                    }
                    other => {
                        self.update(other);
                    }
                }
            }
        });

        tokio::select! {
            e = render_thread => {
                match e {
                    Ok(Event::Quit) => {
                        TUI::exit().unwrap();
                        return false;
                    }
                    Ok(Event::Restart) => {
                        TUI::exit().unwrap();
                        info!("Connection lost");
                        return true;
                    }
                    _ => {info!("Render thread exited")}
                }
            },
        }

        info!("Exiting UI");
        false
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::Input(c) => {
                self.model.text_area.push(c);
            }
            Event::Refresh => {}
            Event::Quit => {
                unreachable!("Quit event should be handled in run()");
            }
            Event::Restart => {
                unreachable!("Restart event should be handled in run()");
            }
            Event::Resize { width, height } => {
                self.tui.resize(width, height);
            }
            Event::ReciveMessage(msg) => {
                self.model.messages.push(msg);
            }
            Event::Send => {
                if self.model.text_area.is_empty() {
                    return;
                }
                let msg = ChatMessage {
                    username: self.user_name.clone(),
                    message: self.model.text_area.clone(),
                };

                self.model.text_area.clear();
                self.ws.send(msg.serialize()).unwrap();
            }
            Event::Backspace => {
                self.model.text_area.pop();
            }
        };
    }

    async fn wait_for_websocket(url: &str) -> Websocket {
        loop {
            let connection = Websocket::connect(url).await;
            match connection {
                Ok(c) => {
                    return c;
                }
                Err(e) => {
                    error!("Failed to connect to server: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
}
