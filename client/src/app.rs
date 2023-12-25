use std::{
    collections::VecDeque,
    sync::{atomic, Arc, Mutex},
};

use futures_util::{select, FutureExt};
use log::info;
use ratatui::widgets::RenderDirection;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::{
    input::EventHandler,
    model::{ChatMessage, Model},
    tui::{TUIMessage, TUI},
    websocket::Websocket,
    Event,
};

pub struct Application {
    pub url: String,
    pub input: EventHandler,
    pub tui: TUI,
    pub model: Model,
    pub ws: Websocket,
}

impl Application {
    pub fn new(ws_url: &str) -> Self {
        let mut t = TUI::new();
        TUI::initialize_panic_handler();
        t.enter().unwrap();

        Self {
            url: ws_url.to_string(),
            tui: TUI::new(),
            input: EventHandler::new(),
            model: Model::default(),
            ws: Websocket::loopback(),
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        info!("Starting Application");

        self.ws = Websocket::connect(&self.url).await.unwrap();

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
                        Event::ReciveMessage(ChatMessage {
                            username: "other person".to_string(),
                            message: msg.unwrap(),
                        })
                    }
                };

                match event {
                    Event::Quit => {
                        break;
                    }
                    other => {
                        self.update(other);
                    }
                }
            }
        });

        tokio::select! {
            _ = render_thread => info!("Render thread exited"),
        }

        info!("Exiting Application");
        Ok(())
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::Input(c) => match c {
                'c' => {
                    self.model.counter += 1;
                }
                _ => {
                    self.model.text_area.push(c);
                }
            },
            Event::Refresh => {}
            Event::Quit => {
                unreachable!("Quit event should be handled in run()");
            }
            Event::Resize { width, height } => {
                self.tui.resize(width, height);
            }
            Event::ReciveMessage(msg) => {
                self.model.messages.push(msg);
            }
            Event::Send => {
                let txt = self.model.text_area.clone();
                self.model.text_area.clear();
                self.update(Event::ReciveMessage(ChatMessage {
                    username: "me".to_string(),
                    message: txt.clone(),
                }));
                self.ws.send(txt).unwrap();
            }
            Event::Backspace => {
                self.model.text_area.pop();
            }
        };
    }
}
