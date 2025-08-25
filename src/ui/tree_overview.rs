use std::rc::Rc;

use ratatui::layout::{Alignment, Position, Rect};
use ratatui::widgets::{Block, Borders, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use serde_json::Value;
use tui_tree_widget::TreeState;
use tui_tree_widget::{Tree as TreeWidget, TreeItem};

use crate::config::keys::Action;
use crate::config::Config;
use crate::parse::Parser;
use crate::tree::{ItemValue, Tree};
use crate::ui::app::ScrollDirection;
use crate::ui::filter::FilterOptions;

pub struct TreeOverview {
    cfg: Rc<Config>,
    state: Option<TreeState<String>>,
    tree: Option<Tree>,
    filter_items: Option<Vec<TreeItem<'static, String>>>,
    before_filter_state: Option<TreeState<String>>,
    last_switches: Vec<(Tree, TreeState<String>)>,
    root_switch: Option<(Tree, TreeState<String>)>,
    root_identifies: Vec<String>,
}

impl TreeOverview {
    pub fn new(cfg: Rc<Config>, tree: Tree) -> Self {
        Self {
            cfg,
            state: Some(TreeState::default()),
            tree: Some(tree),
            filter_items: None,
            before_filter_state: None,
            last_switches: vec![],
            root_switch: None,
            root_identifies: vec![],
        }
    }

    pub fn get_selected(&self) -> Option<String> {
        let selected = self.state().selected();
        if selected.is_empty() {
            return None;
        }
        Some(selected.join("/"))
    }

    pub fn get_root_identifies(&self) -> &[String] {
        self.root_identifies.as_ref()
    }

    pub fn get_value(&self, id: &str) -> Option<Rc<ItemValue>> {
        self.tree().get_value(id)
    }

    pub fn get_parser(&self) -> Rc<Box<dyn Parser>> {
        self.tree().get_parser()
    }

    pub fn on_key(&mut self, action: Action) -> bool {
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
            Action::ExpandChildren => self.expand_children(),
            Action::ExpandAll => self.expand_all(),
            Action::Reset => self.reset(),
            _ => false,
        }
    }

    fn change_root(&mut self) -> bool {
        let Some(id) = self.get_selected() else {
            return false;
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

        let new_tree = Tree::from_value(self.cfg.clone(), value, self.tree().get_parser());

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

    fn expand_children(&mut self) -> bool {
        let selected = self.state().selected();
        if selected.is_empty() {
            return false;
        }
        let selected = selected.join("/");

        let mut state = self.state.take().unwrap();

        for item in &self.tree().items {
            let id = item.identifier();
            if id == selected.as_str() && !item.children().is_empty() {
                if state.opened().contains(&vec![item.identifier().clone()]) {
                    Self::close_all_recursively(vec![], item, &mut state);
                } else {
                    Self::expand_all_recursively(vec![], item, &mut state);
                }
                self.state = Some(state);
                return true;
            }
        }

        self.state = Some(state);
        false
    }

    fn expand_all(&mut self) -> bool {
        let mut state = self.state.take().unwrap();
        for item in &self.tree().items {
            if state.opened().contains(&vec![item.identifier().clone()]) {
                Self::close_all_recursively(vec![], item, &mut state);
            } else {
                Self::expand_all_recursively(vec![], item, &mut state);
            }
        }
        self.state = Some(state);
        true
    }

    fn expand_all_recursively(
        mut id: Vec<String>,
        item: &TreeItem<'static, String>,
        state: &mut TreeState<String>,
    ) {
        id.push(item.identifier().clone());
        state.open(id.clone());

        for child in item.children() {
            Self::expand_all_recursively(id.clone(), child, state);
        }
    }

    fn close_all_recursively(
        mut id: Vec<String>,
        item: &TreeItem<'static, String>,
        state: &mut TreeState<String>,
    ) {
        id.push(item.identifier().clone());
        state.close(&id);
        for child in item.children() {
            Self::close_all_recursively(id.clone(), child, state);
        }
    }

    pub fn on_click(&mut self, column: u16, row: u16) {
        let changed = self.state_mut().click_at(Position { x: column, y: row });
        if !changed {
            self.state_mut().toggle_selected();
        }
    }

    pub fn on_scroll(&mut self, direction: ScrollDirection) -> bool {
        match direction {
            ScrollDirection::Up => self.state_mut().scroll_up(1),
            ScrollDirection::Down => self.state_mut().scroll_down(1),
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
        let mut state = self.state.take().unwrap();
        let items = if let Some(filter_items) = &self.filter_items {
            filter_items
        } else {
            &self.tree().items
        };
        let widget = TreeWidget::new(items)
            .unwrap()
            .experimental_scrollbar(Some(scrollbar))
            .highlight_style(self.cfg.colors.tree.selected.style)
            .block(block);

        frame.render_stateful_widget(widget, area, &mut state);
        self.state = Some(state);
    }

    pub fn begin_filter(&mut self) {
        let state = self.state.take().unwrap();
        self.before_filter_state = Some(state);
        self.state = Some(TreeState::default());
    }

    pub fn end_filter(&mut self) {
        self.filter_items = None;
        let selected = self.state().selected().to_vec();
        let mut state = self.before_filter_state.take().unwrap();
        state.select(selected);
        self.state = Some(state);
    }

    pub fn filter(&mut self, opts: &FilterOptions) {
        let mut state = self.state.take().unwrap();

        let items = &self.tree().items;
        let mut filtered = vec![];

        for item in items {
            if let Some(filtered_item) = self.filter_item(vec![], item, opts, &mut state) {
                filtered.push(filtered_item);
            }
        }

        self.filter_items = Some(filtered);
        self.state = Some(state);
    }

    fn filter_item(
        &self,
        mut id: Vec<String>,
        item: &TreeItem<'static, String>,
        opts: &FilterOptions,
        state: &mut TreeState<String>,
    ) -> Option<TreeItem<'static, String>> {
        id.push(item.identifier().clone());
        let key = id.join("/");
        let item_value = self.tree().get_value(&key).unwrap();

        let ok = opts.filter(&item_value);

        if item.children().is_empty() {
            if ok {
                return Some(item.clone());
            }
            return None;
        }

        let filtered_children: Vec<_> = item
            .children()
            .iter()
            .filter_map(|child| self.filter_item(id.clone(), child, opts, state))
            .collect();

        if filtered_children.is_empty() {
            if ok {
                return Some(TreeItem::new_leaf(
                    item.identifier().clone(),
                    item.text().clone(),
                ));
            }
            return None;
        }

        state.open(id.clone());
        Some(
            TreeItem::new(
                item.identifier().clone(),
                item.text().clone(),
                filtered_children,
            )
            .unwrap(),
        )
    }

    fn tree(&self) -> &Tree {
        self.tree.as_ref().unwrap()
    }

    fn state(&self) -> &TreeState<String> {
        self.state.as_ref().unwrap()
    }

    fn state_mut(&mut self) -> &mut TreeState<String> {
        self.state.as_mut().unwrap()
    }
}
