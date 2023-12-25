use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{CrosstermBackend, Stylize, Terminal as RatatuiTerminal},
    symbols::block,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::io::{stdout, Result, Stdout};

use crate::model::Model;

use log::debug;

type Terminal = RatatuiTerminal<CrosstermBackend<Stdout>>;
pub struct TUI {
    pub terminal: Terminal,
}

pub enum TUIMessage {
    Resize { width: u16, height: u16 },
}

impl TUI {
    pub fn new() -> Self {
        let terminal =
            Terminal::new(CrosstermBackend::new(stdout())).expect("Failed to connect to terminal");
        Self { terminal }
    }

    pub async fn render(&mut self, model: &Model) -> anyhow::Result<()> {
        self.terminal.draw(|frame| TUI::draw(frame, model))?;
        Ok(())
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.terminal.resize(Rect {
            x: 0,
            y: 0,
            width,
            height,
        });
    }

    fn draw(frame: &mut Frame, model: &Model) {
        let area = frame.size();
        frame.render_widget(
            Paragraph::new(format!(
                "Counter: {}",
                model.counter.load(std::sync::atomic::Ordering::SeqCst)
            ))
            .white()
            .on_blue(),
            area,
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(frame.size());

        let t = &model.text_area.lock().unwrap();
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
    pub fn enter(&mut self) -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        self.terminal.clear()?;
        self.terminal.show_cursor()?;
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
    pub fn initialize_panic_handler() {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)
                .unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}
