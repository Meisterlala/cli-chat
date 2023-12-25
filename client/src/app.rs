use std::sync::{atomic, Mutex};

use log::info;
use ratatui::widgets::RenderDirection;

use crate::{
    input::EventHandler,
    model::Model,
    tui::{TUIMessage, TUI},
    websocket::Websocket,
    Event,
};

pub struct Application {
    pub url: String,
    pub input: EventHandler,
    pub tui: TUI,
    pub model: Model,
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
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let mut ws = Websocket::new(&self.url).await;

        let echo_thread = tokio::spawn(async move {
            while let Ok(msg) = ws.recieve().await {
                info!("Received: {}", msg.trim());
            }
        });

        let render_thread = tokio::spawn(async move {
            loop {
                self.tui.render(&self.model).await.ok().or_else(|| {
                    info!("Failed to render");
                    None
                });

                let event = self.input.next().await;
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
            _ = echo_thread => info!("Echo thread exited"),
            _ = render_thread => info!("Render thread exited"),
        }

        info!("Exiting Application");
        Ok(())
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::Input(c) => match c {
                'c' => {
                    self.model
                        .counter
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
                _ => {
                    self.model.text_area.lock().unwrap().push(c);
                }
            },
            Event::Refresh => {}
            Event::Quit => {
                unreachable!("Quit event should be handled in run()");
            }
            Event::Resize { width, height } => {
                self.tui.resize(width, height);
            }
            Event::Send => {}
            Event::Backspace => {
                self.model.text_area.lock().unwrap().pop();
            }
        };
    }
}
