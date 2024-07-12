use std::rc::Rc;

use ratatui::layout::{Alignment, Position, Rect};
use ratatui::widgets::{Block, Borders, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use serde_json::Value;
use tui_tree_widget::Tree as TreeWidget;
use tui_tree_widget::TreeState;

use crate::config::keys::Action;
use crate::config::Config;
use crate::parse::Parser;
use crate::tree::{ItemValue, Tree};
use crate::ui::app::ScrollDirection;

pub(super) struct TreeOverview<'a> {
    cfg: &'a Config,
    state: Option<TreeState<String>>,
    tree: Option<Tree<'a>>,
    last_switches: Vec<(Tree<'a>, TreeState<String>)>,
    root_switch: Option<(Tree<'a>, TreeState<String>)>,
    root_identifies: Vec<String>,
}

impl<'a> TreeOverview<'a> {
    pub(super) fn new(cfg: &'a Config, tree: Tree<'a>) -> Self {
        Self {
            cfg,
            state: Some(TreeState::default()),
            tree: Some(tree),
            last_switches: vec![],
            root_switch: None,
            root_identifies: vec![],
        }
    }

    pub(super) fn get_selected(&self) -> Option<String> {
        let selected = self.state().selected();
        if selected.is_empty() {
            return None;
        }
        Some(selected.join("/"))
    }

    pub(super) fn get_root_identifies(&self) -> &[String] {
        self.root_identifies.as_ref()
    }

    pub(super) fn get_value(&self, id: &str) -> Option<Rc<ItemValue>> {
        self.tree().get_value(id)
    }

    pub(super) fn get_parser(&self) -> Rc<Box<dyn Parser>> {
        self.tree().get_parser()
    }

    pub(super) fn on_key(&mut self, action: Action) -> bool {
        match action {
            Action::MoveUp => self.state_mut().key_up(),
            Action::MoveDown => self.state_mut().key_down(),
            Action::SelectFocus => self.state_mut().toggle_selected(),
            Action::SelectParent => self.select_parent(),
            Action::CloseParent => self.close_parent(),
            Action::PageUp => self.state_mut().scroll_up(3),
            Action::PageDown => self.state_mut().scroll_down(3),
            Action::SelectFirst => self.state_mut().select_first(),
            Action::SelectLast => self.state_mut().select_last(),
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

        let value = match self.tree().get_value(id.as_str()) {
            Some(item) => {
                if !matches!(item.value, Value::Array(_) | Value::Object(_)) {
                    // We don't allow to change root to non-expandable value
                    return false;
                }
                item.value.clone()
            }
            None => return false,
        };

        let new_tree = Tree::from_value(self.cfg, value, self.tree().get_parser());

        let current_tree = self.tree.take().unwrap();
        let current_state = self.state.take().unwrap();
        let switch = (current_tree, current_state);

        if self.root_switch.is_none() {
            self.root_switch = Some(switch);
        } else {
            self.last_switches.push(switch);
        }

        self.root_identifies.push(id);
        self.state = Some(TreeState::default());
        self.tree = Some(new_tree);

        true
    }

    fn reset(&mut self) -> bool {
        let (reset_tree, reset_state) = match self.last_switches.pop() {
            Some((tree, state)) => (tree, state),
            None => match self.root_switch.take() {
                Some(tree) => tree,
                None => {
                    if self.get_selected().is_none() {
                        return false;
                    }
                    self.state = Some(TreeState::default());
                    return true;
                }
            },
        };

        self.root_identifies.pop();
        self.tree = Some(reset_tree);
        self.state = Some(reset_state);

        true
    }

    fn close_parent(&mut self) -> bool {
        if !self.select_parent() {
            return false;
        }

        self.state_mut().toggle_selected()
    }

    fn select_parent(&mut self) -> bool {
        if let Some(parent) = self.get_selected_parent() {
            self.state_mut().select(parent);
            return true;
        }
        false
    }

    fn get_selected_parent(&self) -> Option<Vec<String>> {
        let selected = self.state().selected();
        if selected.len() <= 1 {
            return None;
        }
        // The parent is the selected path without the last element
        let parent: Vec<_> = selected.iter().take(selected.len() - 1).cloned().collect();

        Some(parent)
    }

    pub(super) fn on_click(&mut self, column: u16, row: u16) {
        let changed = self.state_mut().click_at(Position { x: column, y: row });
        if !changed {
            self.state_mut().toggle_selected();
        }
    }

    pub(super) fn on_scroll(&mut self, direction: ScrollDirection) -> bool {
        match direction {
            ScrollDirection::Up => self.state_mut().scroll_up(1),
            ScrollDirection::Down => self.state_mut().scroll_down(1),
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
        let mut state = self.state.take().unwrap();
        let widget = TreeWidget::new(&self.tree().items)
            .unwrap()
            .experimental_scrollbar(Some(scrollbar))
            .highlight_style(self.cfg.colors.tree.selected.style)
            .block(block);

        frame.render_stateful_widget(widget, area, &mut state);
        self.state = Some(state);
    }

    fn tree(&self) -> &Tree<'a> {
        self.tree.as_ref().unwrap()
    }

    fn state(&self) -> &TreeState<String> {
        self.state.as_ref().unwrap()
    }

    fn state_mut(&mut self) -> &mut TreeState<String> {
        self.state.as_mut().unwrap()
    }
}
