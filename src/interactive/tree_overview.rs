use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::{Block, BorderType, Borders, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use tui_tree_widget::Tree as TreeWidget;
use tui_tree_widget::TreeState;

use crate::config::Config;
use crate::tree::Tree;

pub struct TreeOverview<'a> {
    pub cfg: &'a Config,
    pub state: TreeState<String>,
    pub tree: Tree<'a>,
}

impl<'a> TreeOverview<'a> {
    pub fn new(cfg: &'a Config, tree: Tree<'a>) -> Self {
        Self {
            cfg,
            state: TreeState::default(),
            tree,
        }
    }
    pub fn get_focus(&self) -> Option<String> {
        let selected = self.state.selected();
        if selected.is_empty() {
            return None;
        }
        Some(selected.join("/"))
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(None);
        let block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::TOP.union(Borders::RIGHT))
            .border_style(self.cfg.colors.tree.border.style)
            .title_alignment(Alignment::Center)
            .title("Tree Overview");
        let widget = TreeWidget::new(self.tree.items.clone())
            .unwrap()
            .experimental_scrollbar(Some(scrollbar))
            .highlight_style(self.cfg.colors.tree.focus.style)
            .block(block);

        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}
