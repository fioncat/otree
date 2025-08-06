use std::rc::Rc;

use crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use tui_textarea::TextArea;

use crate::config::Config;

pub(super) struct Filter {
    cfg: Rc<Config>,
    text_area: TextArea<'static>,
}

impl Filter {
    pub fn new(cfg: Rc<Config>) -> Self {
        let text_area = TextArea::default();
        Self { cfg, text_area }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        self.text_area.input(key);
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
