use anyhow::Result;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, KeyEvent, MouseButton, MouseEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::{Frame, Terminal};

use crate::config::keys::Action;
use crate::config::Config;
use crate::interactive::data_block::DataBlock;
use crate::interactive::tree_overview::TreeOverview;
use crate::tree::Tree;

enum Refresh {
    /// Update the TUI
    Update,
    /// Skip the update of the TUI
    Skip,
    /// Quit the TUI and return to the shell
    Quit,
}

enum ElementInFocus {
    TreeOverview,
    DataBlock,
}

pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

pub enum ScrollDirection {
    Up,
    Down,
}

pub struct App<'a> {
    cfg: &'a Config,

    focus: ElementInFocus,

    tree_overview: TreeOverview<'a>,
    tree_overview_area: Rect,

    data_block: DataBlock<'a>,
    data_block_area: Rect,

    layout_direction: LayoutDirection,
}

impl<'a> App<'a> {
    pub fn new(cfg: &'a Config, tree: Tree<'a>, layout_direction: LayoutDirection) -> Self {
        Self {
            cfg,
            focus: ElementInFocus::TreeOverview,
            tree_overview: TreeOverview::new(cfg, tree),
            tree_overview_area: Rect::default(),
            data_block: DataBlock::new(cfg),
            data_block_area: Rect::default(),
            layout_direction,
        }
    }

    pub fn show(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        terminal.draw(|frame| self.draw(frame))?;

        loop {
            let refresh = match crossterm::event::read()? {
                Event::Key(key) => self.on_key(key),
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        self.on_click(mouse.column, mouse.row)
                    }
                    MouseEventKind::ScrollUp => {
                        self.on_scroll(ScrollDirection::Up, mouse.column, mouse.row)
                    }
                    MouseEventKind::ScrollDown => {
                        self.on_scroll(ScrollDirection::Down, mouse.column, mouse.row)
                    }
                    _ => Refresh::Skip,
                },
                _ => Refresh::Skip,
            };

            match refresh {
                Refresh::Update => {}
                Refresh::Skip => continue,
                Refresh::Quit => {
                    // restore terminal
                    disable_raw_mode()?;
                    crossterm::execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    return Ok(());
                }
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.refresh_area(frame);

        let selected = self.tree_overview.get_selected();
        if let Some(id) = selected {
            if let Some(data) = self.tree_overview.get_data(id.as_str()) {
                self.data_block.update_data(data);
            }
        }

        let tree_focus = matches!(self.focus, ElementInFocus::TreeOverview);
        self.tree_overview
            .draw(frame, self.tree_overview_area, tree_focus);

        let data_focus = matches!(self.focus, ElementInFocus::DataBlock);
        self.data_block
            .draw(frame, self.data_block_area, data_focus);
    }

    fn refresh_area(&mut self, frame: &Frame) {
        match self.layout_direction {
            LayoutDirection::Vertical => {
                let vertical =
                    Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
                [self.tree_overview_area, self.data_block_area] = vertical.areas(frame.size());
            }
            LayoutDirection::Horizontal => {
                let chunks =
                    Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(frame.size());
                self.tree_overview_area = chunks[0];
                self.data_block_area = chunks[1];
            }
        }
    }

    fn can_switch_to_data_block(&self) -> bool {
        match self.focus {
            ElementInFocus::TreeOverview => self.tree_overview.get_selected().is_some(),
            ElementInFocus::DataBlock => false,
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> Refresh {
        let action = self.cfg.keys.get_key_action(key.code);
        if action.is_none() {
            return Refresh::Skip;
        }
        let action = action.unwrap();

        if let Action::Quit = action {
            return Refresh::Quit;
        }

        if let Action::Switch = action {
            match self.focus {
                ElementInFocus::TreeOverview if self.can_switch_to_data_block() => {
                    self.focus = ElementInFocus::DataBlock;
                    return Refresh::Update;
                }
                ElementInFocus::DataBlock => {
                    self.focus = ElementInFocus::TreeOverview;
                    return Refresh::Update;
                }
                _ => return Refresh::Skip,
            }
        }

        let update = match self.focus {
            ElementInFocus::TreeOverview => self.tree_overview.on_key(action),
            ElementInFocus::DataBlock => self.data_block.on_key(action),
        };

        if update {
            Refresh::Update
        } else {
            Refresh::Skip
        }
    }

    fn on_click(&mut self, column: u16, row: u16) -> Refresh {
        if let Some(index) = Self::get_row_inside(column, row, self.tree_overview_area) {
            self.tree_overview.on_click(index);
            self.focus = ElementInFocus::TreeOverview;
            return Refresh::Update;
        }

        if Self::get_row_inside(column, row, self.data_block_area).is_some() {
            return if self.can_switch_to_data_block() {
                self.focus = ElementInFocus::DataBlock;
                Refresh::Update
            } else {
                Refresh::Skip
            };
        }

        Refresh::Skip
    }

    fn on_scroll(&mut self, direction: ScrollDirection, column: u16, row: u16) -> Refresh {
        let update = if Self::get_row_inside(column, row, self.tree_overview_area).is_some() {
            self.tree_overview.on_scroll(direction)
        } else if Self::get_row_inside(column, row, self.data_block_area).is_some() {
            self.data_block.on_scroll(direction)
        } else {
            false
        };

        if update {
            Refresh::Update
        } else {
            Refresh::Skip
        }
    }

    fn get_row_inside(column: u16, row: u16, area: Rect) -> Option<u16> {
        if area.contains(Position { x: column, y: row }) {
            Some(row.saturating_sub(area.top()).saturating_sub(1))
        } else {
            None
        }
    }
}
