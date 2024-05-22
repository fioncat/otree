use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::{Block, Borders, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use serde_json::Value;
use tui_tree_widget::Tree as TreeWidget;
use tui_tree_widget::TreeState;

use crate::config::keys::Action;
use crate::config::Config;
use crate::tree::Tree;
use crate::ui::app::ScrollDirection;

pub(super) struct TreeOverview<'a> {
    cfg: &'a Config,
    state: TreeState<String>,
    tree: Option<Tree<'a>>,
    last_trees: Vec<Tree<'a>>,
    root_tree: Option<Tree<'a>>,
}

impl<'a> TreeOverview<'a> {
    pub(super) fn new(cfg: &'a Config, tree: Tree<'a>) -> Self {
        Self {
            cfg,
            state: TreeState::default(),
            tree: Some(tree),
            last_trees: vec![],
            root_tree: None,
        }
    }

    pub(super) fn get_selected(&self) -> Option<String> {
        let selected = self.state.get_selected();
        if selected.is_empty() {
            return None;
        }

        Some(selected.join("/"))
    }

    pub(super) fn get_data(&self, id: &str) -> Option<String> {
        self.tree().details.get(id).map(|d| d.value.clone())
    }

    pub(super) fn on_key(&mut self, action: Action) -> bool {
        match action {
            Action::MoveUp => self.state.key_up(),
            Action::MoveDown => self.state.key_down(),
            Action::SelectFocus => self.state.toggle_selected(),
            Action::SelectParent => self.select_parent(),
            Action::CloseParent => self.close_parent(),
            Action::PageUp => self.state.scroll_up(3),
            Action::PageDown => self.state.scroll_down(3),
            Action::SelectFirst => self.state.select_first(),
            Action::SelectLast => self.state.select_last(),
            Action::ChangeRoot => self.change_root(),
            Action::Reset => self.reset(),
            _ => false,
        }
    }

    fn change_root(&mut self) -> bool {
        let id = match self.get_selected() {
            Some(id) => id,
            None => return false,
        };

        let value = match self.tree().details.get(id.as_str()) {
            Some(detail) => {
                if !matches!(detail.raw_value, Value::Array(_) | Value::Object(_)) {
                    // We donot allow to change root to non-expandable value
                    return false;
                }
                detail.raw_value.clone()
            }
            None => return false,
        };

        let new_tree = Tree::from_value(self.cfg, value, self.tree().content_type)
            .expect("build tree from change_root must success");

        let current_tree = self.tree.take().unwrap();
        if self.root_tree.is_none() {
            self.root_tree = Some(current_tree);
        } else {
            self.last_trees.push(current_tree);
        }

        self.state = TreeState::default();
        self.tree = Some(new_tree);

        true
    }

    fn reset(&mut self) -> bool {
        let reset_tree = match self.last_trees.pop() {
            Some(tree) => tree,
            None => match self.root_tree.take() {
                Some(tree) => tree,
                None => {
                    if self.get_selected().is_none() {
                        return false;
                    }
                    self.state = TreeState::default();
                    return true;
                }
            },
        };

        self.state = TreeState::default();
        self.tree = Some(reset_tree);

        true
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
        let selected = self.state.get_selected();
        if selected.len() <= 1 {
            return None;
        }
        // The parent is the selected path without the last element
        let parent: Vec<_> = selected.iter().take(selected.len() - 1).cloned().collect();

        Some(parent)
    }

    pub(super) fn on_click(&mut self, index: u16) {
        let offset = self.state.get_offset();
        let index = (index as usize) + offset;

        let changed = self.state.select_visible_index(index);
        if !changed {
            self.state.toggle_selected();
        }
    }

    pub(super) fn on_scroll(&mut self, direction: ScrollDirection) -> bool {
        match direction {
            ScrollDirection::Up => self.state.scroll_up(1),
            ScrollDirection::Down => self.state.scroll_down(1),
        }
    }

    pub(super) fn draw(&mut self, frame: &mut Frame, area: Rect, focus: bool) {
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
        let widget = TreeWidget::new(&self.tree.as_ref().unwrap().items)
            .unwrap()
            .experimental_scrollbar(Some(scrollbar))
            .highlight_style(self.cfg.colors.tree.selected.style)
            .block(block);

        frame.render_stateful_widget(widget, area, &mut self.state);
    }

    fn tree(&self) -> &Tree<'a> {
        self.tree.as_ref().unwrap()
    }
}
