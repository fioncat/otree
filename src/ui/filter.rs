use std::rc::Rc;

use ratatui::layout::{Alignment, Position, Rect};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use serde_json::Value;

use crate::config::keys::{Action, Key, KeyAction};
use crate::config::Config;
use crate::tree::ItemValue;

pub struct Filter {
    cfg: Rc<Config>,
    text: String,
    cursor: usize,
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
        let ignore_case = cfg.filter.ignore_case;
        Self {
            cfg,
            text: String::new(),
            cursor: 0,
            target,
            ignore_case,
        }
    }

    pub fn on_key(&mut self, ka: KeyAction) -> FilterAction {
        if let Key::Char(c) = ka.key {
            self.insert_char(c);
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
                self.delete_char();
                FilterAction::Edit
            }
            Action::MoveLeft => {
                self.move_left();
                FilterAction::Edit
            }
            Action::MoveRight => {
                self.move_right();
                FilterAction::Edit
            }
            Action::SelectFirst => {
                self.cursor = 0;
                FilterAction::Edit
            }
            Action::SelectLast => {
                self.cursor = self.text.chars().count();
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
        self.text.trim().to_string()
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

        let widget = Paragraph::new(Text::from(self.text.as_str())).block(block);
        frame.render_widget(widget, area);

        if focus {
            let inner_width = area.width.saturating_sub(2);
            let cursor = u16::try_from(self.cursor)
                .unwrap_or(u16::MAX)
                .min(inner_width);
            frame.set_cursor_position(Position {
                x: area.x.saturating_add(1 + cursor),
                y: area.y.saturating_add(1),
            });
        }
    }

    fn insert_char(&mut self, c: char) {
        let byte_idx = Self::char_to_byte_idx(&self.text, self.cursor);
        self.text.insert(byte_idx, c);
        self.cursor += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let byte_idx = Self::char_to_byte_idx(&self.text, self.cursor - 1);
        self.text.remove(byte_idx);
        self.cursor -= 1;
    }

    fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_right(&mut self) {
        self.cursor = self.cursor.saturating_add(1).min(self.text.chars().count());
    }

    fn char_to_byte_idx(text: &str, char_idx: usize) -> usize {
        text.char_indices()
            .nth(char_idx)
            .map_or(text.len(), |(idx, _)| idx)
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
