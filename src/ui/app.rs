use anyhow::Result;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, KeyEvent, MouseButton, MouseEventKind,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::{Frame, Terminal};

use crate::config::keys::Action;
use crate::config::{Config, LayoutDirection};
use crate::tree::Tree;
use crate::ui::data_block::DataBlock;
use crate::ui::tree_overview::TreeOverview;

enum Refresh {
    /// Update the TUI
    Update,
    /// Skip the update of the TUI
    Skip,
    /// Quit the TUI and return to the shell
    Quit,
}

#[derive(Debug, Clone, Copy)]
enum ElementInFocus {
    TreeOverview,
    DataBlock,
    None,
}

pub enum ScrollDirection {
    Up,
    Down,
}

pub struct App<'a> {
    cfg: &'a Config,

    focus: ElementInFocus,
    last_focus: Option<ElementInFocus>,

    tree_overview: TreeOverview<'a>,
    tree_overview_area: Rect,

    data_block: DataBlock<'a>,
    data_block_area: Rect,

    layout_direction: LayoutDirection,
    layout_tree_size: u16,
}

impl<'a> App<'a> {
    pub fn new(cfg: &'a Config, tree: Tree<'a>) -> Self {
        Self {
            cfg,
            focus: ElementInFocus::TreeOverview,
            last_focus: None,
            tree_overview: TreeOverview::new(cfg, tree),
            tree_overview_area: Rect::default(),
            data_block: DataBlock::new(cfg),
            data_block_area: Rect::default(),
            layout_direction: cfg.layout.direction,
            layout_tree_size: cfg.layout.tree_size,
        }
    }

    pub fn show(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
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
                // When resize happens, we need to redraw the widgets to fit the new size
                Event::Resize(_, _) => Refresh::Update,
                Event::FocusGained => self.on_focus_changed(true),
                Event::FocusLost => self.on_focus_changed(false),
                _ => Refresh::Skip,
            };

            match refresh {
                Refresh::Update => {}
                Refresh::Skip => continue,
                Refresh::Quit => {
                    // restore terminal
                    terminal::disable_raw_mode()?;
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
                self.data_block.update_data(data, self.data_block_area);
            }
            // TODO: When we cannot find data, should warn user (maybe message in data block?)
        }

        let tree_focus = matches!(self.focus, ElementInFocus::TreeOverview);
        self.tree_overview
            .draw(frame, self.tree_overview_area, tree_focus);

        let data_focus = matches!(self.focus, ElementInFocus::DataBlock);
        self.data_block
            .draw(frame, self.data_block_area, data_focus);
    }

    fn refresh_area(&mut self, frame: &Frame) {
        let tree_size = self.layout_tree_size;
        let data_size = 100 - tree_size;

        match self.layout_direction {
            LayoutDirection::Vertical => {
                let vertical = Layout::vertical([
                    Constraint::Percentage(tree_size),
                    Constraint::Percentage(data_size),
                ]);
                [self.tree_overview_area, self.data_block_area] = vertical.areas(frame.size());
            }
            LayoutDirection::Horizontal => {
                let horizontal = Layout::horizontal([
                    Constraint::Percentage(tree_size),
                    Constraint::Percentage(data_size),
                ]);
                [self.tree_overview_area, self.data_block_area] = horizontal.areas(frame.size());
            }
        }
    }

    fn can_switch_to_data_block(&self) -> bool {
        match self.focus {
            ElementInFocus::TreeOverview => self.tree_overview.get_selected().is_some(),
            ElementInFocus::None => true,
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

        if let Action::ChangeLayout = action {
            match self.layout_direction {
                LayoutDirection::Vertical => self.layout_direction = LayoutDirection::Horizontal,
                LayoutDirection::Horizontal => self.layout_direction = LayoutDirection::Vertical,
            }
            return Refresh::Update;
        }

        if let Action::TreeScaleUp = action {
            if self.layout_tree_size == Config::MAX_LAYOUT_TREE_SIZE {
                return Refresh::Skip;
            }

            self.layout_tree_size += 2;
            if self.layout_tree_size > Config::MAX_LAYOUT_TREE_SIZE {
                self.layout_tree_size = Config::MAX_LAYOUT_TREE_SIZE;
            }

            return Refresh::Update;
        }

        if let Action::TreeScaleDown = action {
            if self.layout_tree_size < Config::MIN_LAYOUT_TREE_SIZE {
                return Refresh::Skip;
            }

            self.layout_tree_size = self.layout_tree_size.saturating_sub(2);
            if self.layout_tree_size < Config::MIN_LAYOUT_TREE_SIZE {
                self.layout_tree_size = Config::MIN_LAYOUT_TREE_SIZE;
            }

            return Refresh::Update;
        }

        let update = match self.focus {
            ElementInFocus::TreeOverview => self.tree_overview.on_key(action),
            ElementInFocus::DataBlock => self.data_block.on_key(action),
            ElementInFocus::None => return Refresh::Skip,
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

    fn on_focus_changed(&mut self, focus: bool) -> Refresh {
        if focus {
            if !matches!(self.focus, ElementInFocus::None) {
                // We are already focused, no need to update
                return Refresh::Skip;
            }

            // This implements the functionality to refocus on the last focused widget when
            // we return after losing focus.
            let last_focus = self
                .last_focus
                .take()
                .unwrap_or(ElementInFocus::TreeOverview);
            self.focus = last_focus;
            return Refresh::Update;
        }

        if matches!(self.focus, ElementInFocus::None) {
            // We are already not focused, no need to update
            return Refresh::Skip;
        }

        self.last_focus = Some(self.focus);
        self.focus = ElementInFocus::None;

        Refresh::Update
    }

    fn get_row_inside(column: u16, row: u16, area: Rect) -> Option<u16> {
        if area.contains(Position { x: column, y: row }) {
            Some(row.saturating_sub(area.top()).saturating_sub(1))
        } else {
            None
        }
    }
}
