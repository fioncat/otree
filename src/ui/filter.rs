use std::rc::Rc;

use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use tui_textarea::{CursorMove, TextArea};

use crate::config::keys::{Action, Key, KeyAction};
use crate::config::Config;

pub(super) struct Filter {
    cfg: Rc<Config>,
    text_area: TextArea<'static>,
}

pub(super) enum FilterAction {
    Edit,
    Confirm,
    Skip,
    Quit,
}

impl Filter {
    pub fn new(cfg: Rc<Config>) -> Self {
        let text_area = TextArea::default();
        Self { cfg, text_area }
    }

    pub fn on_key(&mut self, ka: KeyAction) -> FilterAction {
        if let Key::Char(c) = ka.key {
            self.text_area.insert_char(c);
            return FilterAction::Edit;
        }

        let action = match ka.action {
            Some(a) => a,
            None => return FilterAction::Skip,
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

    pub fn get_text(&self) -> String {
        let lines = self.text_area.lines();
        if lines.is_empty() {
            return String::new();
        }
        // There won't be more than one line
        lines[0].trim().to_string()
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, focus: bool) {
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
            .title("Filter");
        self.text_area.set_block(block);

        frame.render_widget(&self.text_area, area);
    }
}
