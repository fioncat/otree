use std::collections::HashMap;

use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::{Block, BorderType, Borders, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use tui_tree_widget::Tree as TreeWidget;
use tui_tree_widget::TreeState;

use crate::config::keys::Action;
use crate::config::Config;
use crate::interactive::app::ScrollDirection;
use crate::tree::{Detail, Tree};

pub struct TreeOverview<'a> {
    cfg: &'a Config,
    state: TreeState<String>,
    tree: Tree<'a>,
}

impl<'a> TreeOverview<'a> {
    pub fn new(cfg: &'a Config, tree: Tree<'a>) -> Self {
        Self {
            cfg,
            state: TreeState::default(),
            tree,
        }
    }

    pub fn get_selected(&self) -> Option<String> {
        let selected = self.state.selected();
        if selected.is_empty() {
            return None;
        }

        Some(selected.join("/"))
    }

    pub fn get_data(&self, id: &str) -> Option<String> {
        self.tree.details.get(id).map(|d| d.value.clone())
    }

    pub fn get_details(&self) -> HashMap<String, Detail> {
        self.tree.details.clone()
    }

    pub fn on_key(&mut self, action: Action) -> bool {
        match action {
            Action::MoveUp => self.move_up(),
            Action::MoveDown => self.move_down(),
            Action::SelectFocus => self.toggle_selected(),
            Action::PageUp => self.scroll_up(3),
            Action::PageDown => self.scroll_down(3),
            Action::SelectFirst => self.select_first(),
            Action::SelectLast => self.select_last(),
            Action::SelectParent => false,
            _ => false,
        }
    }

    pub fn move_up(&mut self) -> bool {
        self.state.key_up(&self.tree.items)
    }

    pub fn move_down(&mut self) -> bool {
        self.state.key_down(&self.tree.items)
    }

    pub fn toggle_selected(&mut self) -> bool {
        self.state.toggle_selected()
    }

    pub fn scroll_up(&mut self, lines: usize) -> bool {
        self.state.scroll_up(lines)
    }

    pub fn scroll_down(&mut self, lines: usize) -> bool {
        self.state.scroll_down(lines)
    }

    pub fn select_first(&mut self) -> bool {
        self.state.select_first(&self.tree.items)
    }

    pub fn select_last(&mut self) -> bool {
        self.state.select_last(&self.tree.items)
    }

    pub fn on_click(&mut self, index: u16) {
        let offset = self.state.get_offset();
        let index = (index as usize) + offset;

        let changed = self.state.select_visible_index(&self.tree.items, index);
        if !changed {
            self.toggle_selected();
        }
    }

    pub fn on_scroll(&mut self, direction: ScrollDirection) -> bool {
        match direction {
            ScrollDirection::Up => self.scroll_up(1),
            ScrollDirection::Down => self.scroll_down(1),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, focus: bool) {
        let border_style = if focus {
            self.cfg.colors.focus_border.style
        } else {
            self.cfg.colors.tree.border.style
        };

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(None);
        let block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_alignment(Alignment::Center)
            .title("Tree Overview");
        let widget = TreeWidget::new(self.tree.items.clone())
            .unwrap()
            .experimental_scrollbar(Some(scrollbar))
            .highlight_style(self.cfg.colors.tree.selected.style)
            .block(block);

        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}
