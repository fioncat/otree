use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::config::keys::Action;
use crate::config::Config;
use crate::interactive::app::ScrollDirection;

pub struct DataBlock<'a> {
    cfg: &'a Config,
    data: String,

    offset_x: u16,
    offset_y: u16,
}

impl<'a> DataBlock<'a> {
    pub fn new(cfg: &'a Config) -> Self {
        Self {
            cfg,
            data: String::new(),
            offset_x: 0,
            offset_y: 0,
        }
    }

    pub fn on_key(&mut self, action: Action) -> bool {
        false
    }

    pub fn on_scroll(&mut self, direction: ScrollDirection) -> bool {
        false
    }

    pub fn update_data(&mut self, data: String) {
        self.data = data;
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, focus: bool) {
        let border_style = if focus {
            self.cfg.colors.focus_border.style
        } else {
            self.cfg.colors.data.border.style
        };

        let block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_alignment(Alignment::Center)
            .title("Data Block");

        let widget = Paragraph::new(self.data.as_str())
            .style(self.cfg.colors.data.text.style)
            .block(block)
            .scroll((self.offset_x, self.offset_y));

        frame.render_widget(widget, area);
    }
}
