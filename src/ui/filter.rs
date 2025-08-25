use std::rc::Rc;

use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use serde_json::Value;
use tui_textarea::{CursorMove, TextArea};

use crate::config::keys::{Action, Key, KeyAction};
use crate::config::Config;
use crate::tree::ItemValue;

pub struct Filter {
    cfg: Rc<Config>,
    text_area: TextArea<'static>,
    target: FilterTarget,
    ignore_case: bool,
}

pub enum FilterAction {
    Edit,
    Confirm,
    Skip,
    Quit,
}

pub struct FilterOptions {
    pub text: String,
    pub target: FilterTarget,
    pub ignore_case: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterTarget {
    Key,
    Value,
    All,
}

impl Filter {
    pub fn new(cfg: Rc<Config>, target: FilterTarget) -> Self {
        let text_area = TextArea::default();
        let ignore_case = cfg.filter.ignore_case;
        Self {
            cfg,
            text_area,
            target,
            ignore_case,
        }
    }

    pub fn on_key(&mut self, ka: KeyAction) -> FilterAction {
        if let Key::Char(c) = ka.key {
            self.text_area.insert_char(c);
            return FilterAction::Edit;
        }

        let Some(action) = ka.action else {
            return FilterAction::Skip;
        };
        match action {
            Action::SelectFocus | Action::Switch => {
                if self.get_text().is_empty() {
                    // no filter text, quit filter mode
                    return FilterAction::Quit;
                }
                FilterAction::Confirm
            }
            Action::CloseParent => {
                self.text_area.delete_char();
                FilterAction::Edit
            }
            Action::MoveLeft => {
                self.text_area.move_cursor(CursorMove::Back);
                FilterAction::Edit
            }
            Action::MoveRight => {
                self.text_area.move_cursor(CursorMove::Forward);
                FilterAction::Edit
            }
            Action::SelectFirst => {
                self.text_area.move_cursor(CursorMove::Head);
                FilterAction::Edit
            }
            Action::SelectLast => {
                self.text_area.move_cursor(CursorMove::End);
                FilterAction::Edit
            }
            Action::Reset => FilterAction::Quit,
            _ => FilterAction::Skip, // I cannot handle this
        }
    }

    pub fn get_options(&self) -> FilterOptions {
        FilterOptions {
            text: self.get_text(),
            target: self.target,
            ignore_case: self.ignore_case,
        }
    }

    pub fn set_target(&mut self, target: FilterTarget) {
        self.target = target;
    }

    pub fn switch_ignore_case(&mut self) {
        self.ignore_case = !self.ignore_case;
    }

    fn get_text(&self) -> String {
        let lines = self.text_area.lines();
        if lines.is_empty() {
            return String::new();
        }
        // There won't be more than one line
        lines[0].trim().to_string()
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, focus: bool) {
        let mut hints = vec![];
        if self.ignore_case {
            hints.push("I");
        }
        match self.target {
            FilterTarget::All => hints.push("*"),
            FilterTarget::Key => hints.push("K"),
            FilterTarget::Value => hints.push("V"),
        }
        let title = if hints.is_empty() {
            String::from("Filter")
        } else {
            format!("Filter ({})", hints.join(","))
        };
        let (border_style, border_type) = super::get_border_style(
            &self.cfg.colors.focus_border,
            &self.cfg.colors.filter.border,
            focus,
        );

        let block = Block::new()
            .border_type(border_type)
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_alignment(Alignment::Center)
            .title(title);
        self.text_area.set_block(block);

        frame.render_widget(&self.text_area, area);
    }
}

impl FilterOptions {
    pub fn filter(&self, item: &ItemValue) -> bool {
        if matches!(self.target, FilterTarget::Key) {
            return self.contains(item.name.as_str());
        }

        let value = match item.value {
            Value::String(ref s) => s.clone(),
            Value::Number(ref n) => n.to_string(),
            _ => String::new(),
        };

        match self.target {
            FilterTarget::Value => self.contains(&value),
            FilterTarget::All => {
                if self.contains(item.name.as_str()) {
                    return true;
                }
                self.contains(&value)
            }
            FilterTarget::Key => unreachable!(),
        }
    }

    fn contains(&self, text: &str) -> bool {
        if self.text.is_empty() || text.is_empty() {
            return false;
        }
        if self.ignore_case {
            text.to_lowercase()
                .contains(self.text.to_lowercase().as_str())
        } else {
            text.contains(&self.text)
        }
    }
}
