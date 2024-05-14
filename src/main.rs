#![allow(dead_code)]

mod config;
mod interactive;
mod tree;

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use config::keys::Action;
use crossterm::event::{Event, MouseEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::config::Config;
use crate::interactive::tree_overview::TreeOverview;
use crate::tree::{ContentType, Tree};

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        bail!("invalid usage");
    }

    let path = args.get(1).unwrap();
    let content_type = if path.ends_with(".toml") {
        ContentType::Toml
    } else if path.ends_with(".json") {
        ContentType::Json
    } else {
        bail!("unsupported file type");
    };
    let path = PathBuf::from(path);

    let data = fs::read(path).context("read file")?;
    let data = String::from_utf8(data).context("parse utf8")?;

    let mut cfg = Config::default();
    cfg.parse().context("parse config")?;

    let tree = Tree::parse(&cfg, &data, content_type)?;
    let widget = TreeOverview::new(&cfg, tree);
    draw(&cfg, widget)?;

    Ok(())
}

fn draw(cfg: &Config, mut widget: TreeOverview) -> Result<()> {
    // Terminal initialization
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    terminal.draw(|frame| widget.draw(frame, frame.size()))?;
    loop {
        let update = match crossterm::event::read()? {
            Event::Key(key) => {
                let action = cfg.keys.get_key_action(key.code);
                match action {
                    Some(action) => match action {
                        Action::MoveUp => widget.state.key_up(&widget.tree.items),
                        Action::MoveDown => widget.state.key_down(&widget.tree.items),
                        Action::SelectFocus => widget.state.toggle_selected(),
                        Action::PageUp => widget.state.scroll_up(3),
                        Action::PageDown => widget.state.scroll_down(3),
                        Action::SelectFirst => widget.state.select_first(&widget.tree.items),
                        Action::SelectLast => widget.state.select_last(&widget.tree.items),
                        Action::SelectParent => false,
                        Action::Quit => {
                            // restore terminal
                            crossterm::terminal::disable_raw_mode()?;
                            crossterm::execute!(
                                terminal.backend_mut(),
                                crossterm::terminal::LeaveAlternateScreen,
                                crossterm::event::DisableMouseCapture
                            )?;
                            terminal.show_cursor()?;
                            return Ok(());
                        }
                    },
                    None => false,
                }
            }

            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => widget.state.scroll_down(1),
                MouseEventKind::ScrollUp => widget.state.scroll_up(1),
                _ => false,
            },
            _ => false,
        };
        if update {
            terminal.draw(|frame| widget.draw(frame, frame.size()))?;
        }
    }
}
