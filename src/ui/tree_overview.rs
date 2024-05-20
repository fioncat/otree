use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::{Block, Borders, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use tui_tree_widget::Tree as TreeWidget;
use tui_tree_widget::TreeState;

use crate::config::keys::Action;
use crate::config::Config;
use crate::tree::Tree;
use crate::ui::app::ScrollDirection;

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

    pub fn on_key(&mut self, action: Action) -> bool {
        match action {
            Action::MoveUp => self.state.key_up(&self.tree.items),
            Action::MoveDown => self.state.key_down(&self.tree.items),
            Action::SelectFocus => self.state.toggle_selected(),
            Action::SelectParent => self.select_parent(),
            Action::CloseParent => self.close_parent(),
            Action::PageUp => self.state.scroll_up(3),
            Action::PageDown => self.state.scroll_down(3),
            Action::SelectFirst => self.state.select_first(&self.tree.items),
            Action::SelectLast => self.state.select_last(&self.tree.items),
            _ => false,
        }
    }

    fn close_parent(&mut self) -> bool {
        if !self.select_parent() {
            return false;
        }

        self.state.toggle_selected()
    }

    fn select_parent(&mut self) -> bool {
        if let Some(parent) = self.get_selected_parent() {
            self.state.select(parent);
            return true;
        }
        false
    }

    fn get_selected_parent(&self) -> Option<Vec<String>> {
        let selected = self.state.selected();
        if selected.len() <= 1 {
            return None;
        }
        // The parent is the selected path without the last element
        let parent: Vec<_> = selected
            .clone()
            .into_iter()
            .take(selected.len() - 1)
            .collect();

        Some(parent)
    }

    pub fn on_click(&mut self, index: u16) {
        let offset = self.state.get_offset();
        let index = (index as usize) + offset;

        let changed = self.state.select_visible_index(&self.tree.items, index);
        if !changed {
            self.state.toggle_selected();
        }
    }

    pub fn on_scroll(&mut self, direction: ScrollDirection) -> bool {
        match direction {
            ScrollDirection::Up => self.state.scroll_up(1),
            ScrollDirection::Down => self.state.scroll_down(1),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, focus: bool) {
        let (border_style, border_type) = super::get_border_style(
            &self.cfg.colors.focus_border,
            &self.cfg.colors.tree.border,
            focus,
        );

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(None);
        let block = Block::new()
            .border_type(border_type)
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
