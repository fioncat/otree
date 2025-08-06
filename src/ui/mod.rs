mod app;
mod data_block;
mod filter;
mod footer;
mod header;
mod popup;
mod tree_overview;

use std::io::Stdout;

use anyhow::{Context, Result};
use app::ShowResult;
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

pub fn start(mut app: App) -> Result<()> {
    let mut terminal = new_terminal()?;
    let mut result: Result<()> = Ok(());

    loop {
        let show_result = app.show(&mut terminal);
        if let Err(err) = show_result {
            result = Err(err);
            break;
        }

        match show_result.unwrap() {
            ShowResult::Edit(edit) => {
                restore(&mut terminal)?;
                edit.run();
                terminal = new_terminal()?;
            }
            ShowResult::Quit => break,
        }
    }

    // Regardless of how the TUI app executes, we should always restore the terminal.
    // Otherwise, if the app encounters an error (such as a draw error), the user's terminal
    // will become a mess.
    restore(&mut terminal)?;
    result
}

fn new_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
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

fn restore(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
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
