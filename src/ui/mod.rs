mod app;
mod data_block;
mod header;
mod tree_overview;

use std::io::Stdout;

use anyhow::{Context, Result};
use crossterm::{event, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::style::Style;
use ratatui::widgets::BorderType;
use ratatui::Terminal;

use crate::config::colors::Color;

pub use app::App;
pub use header::HeaderContext;

fn get_border_style(focus_color: &Color, normal_color: &Color, focus: bool) -> (Style, BorderType) {
    let color = if focus { focus_color } else { normal_color };
    let border_type = if color.bold {
        BorderType::Thick
    } else {
        BorderType::Plain
    };

    (color.style, border_type)
}

pub fn start() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    terminal::enable_raw_mode().context("enable terminal raw mode")?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )
    .context("execute terminal commands for stdout")?;

    let terminal = Terminal::new(CrosstermBackend::new(stdout)).context("init terminal")?;
    Ok(terminal)
}

pub fn restore(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal::disable_raw_mode().context("disable terminal raw mode")?;
    crossterm::execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )
    .context("execute terminal commands")?;
    terminal.show_cursor().context("restore terminal cursor")?;
    Ok(())
}
