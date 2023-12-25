use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{CrosstermBackend, Stylize, Terminal},
    symbols::block,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::{
    fmt::format,
    io::{stdout, Result, Stdout},
    sync::Arc,
};
use tokio::{select, sync::mpsc};

use crate::{input, state::State, Event};

use log::{debug, error};

pub struct TUI {
    state: Arc<State>,
}

impl TUI {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }

    pub async fn run(&mut self) -> Result<()> {
        Self::initialize_panic_handler();
        Self::enter()?;

        let mut terminal =
            Terminal::new(CrosstermBackend::new(stdout())).expect("Failed to connect to terminal");
        terminal.clear()?;
        terminal.show_cursor()?;

        let mut events = input::EventHandler::new();

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let next_event = events.next().await;

            // Update State
            // Get mut arc

            self.state.update(&next_event);

            // Have TUI handle event
            match &next_event {
                Event::Input(c) => {
                    if let 'q' = c {
                        break;
                    }
                }
                Event::Refresh => {}
                Event::Quit => break,
                Event::Resize { width, height } => {
                    terminal.resize(Rect::new(0, 0, *width, *height))?;
                }
            };
        }

        Self::exit()?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.size();
        frame.render_widget(
            Paragraph::new(format!(
                "Counter: {}",
                self.state.counter.load(std::sync::atomic::Ordering::SeqCst)
            ))
            .white()
            .on_blue(),
            area,
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(frame.size());

        let t = &self.state.text_area.lock().unwrap();
        let avaliable_space = (layout[1].height - 2) * (layout[1].width - 2);

        let text = if avaliable_space < t.len() as u16 {
            debug!("Text doesnt fits");
            format!("{}{}", &t[..avaliable_space as usize - 1], block::FULL)
        } else {
            t.to_string()
        };

        let block = Block::default().borders(Borders::ALL).title("Input");

        frame.render_widget(
            Paragraph::new(format!("{text}_"))
                .white()
                .on_blue()
                .wrap(Wrap { trim: true })
                .block(block),
            layout[1],
        );
    }

    /// Enter raw mode and the alternate screen
    fn enter() -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(())
    }

    /// Exit raw mode and the alternate screen
    pub fn exit() -> Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    /// Initialize a panic handler that exits raw mode and the alternate screen
    /// before aborting the program
    fn initialize_panic_handler() {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)
                .unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}
