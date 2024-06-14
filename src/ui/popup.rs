use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::config::{keys::Action, Config};

use super::app::ScrollDirection;

#[derive(Debug, Clone, Copy)]
pub(super) enum PopupLevel {
    Error,
}

pub(super) struct Popup<'a> {
    data: Option<(String, PopupLevel)>,

    cfg: &'a Config,

    scroll: usize,
}

impl<'a> Popup<'a> {
    pub(super) fn new(cfg: &'a Config) -> Self {
        Self {
            data: None,
            cfg,
            scroll: 0,
        }
    }

    pub(super) fn set_data(&mut self, data: String, level: PopupLevel) {
        self.data = Some((data, level));
    }

    pub(super) fn on_key(&mut self, action: Action) -> bool {
        match action {
            Action::MoveDown => self.scroll_down(1),
            Action::MoveUp => self.scroll_up(1),
            Action::SelectFocus | Action::Reset => self.disable(),
            _ => false,
        }
    }

    pub(super) fn on_scroll(&mut self, direction: ScrollDirection) -> bool {
        match direction {
            ScrollDirection::Up => self.scroll_up(3),
            ScrollDirection::Down => self.scroll_down(3),
        }
    }

    fn scroll_down(&mut self, lines: usize) -> bool {
        if self.data.is_none() {
            return false;
        }

        self.scroll = self.scroll.saturating_add(lines);
        true
    }

    fn scroll_up(&mut self, lines: usize) -> bool {
        if self.data.is_none() {
            return false;
        }

        if self.scroll == 0 {
            return false;
        }

        self.scroll = self.scroll.saturating_sub(lines);
        true
    }

    pub(super) fn disable(&mut self) -> bool {
        if self.data.is_none() {
            return false;
        }

        self.data = None;
        self.scroll = 0;
        true
    }

    pub(super) fn is_disabled(&self) -> bool {
        self.data.is_none()
    }

    pub(super) fn draw(&self, frame: &mut Frame) {
        let (text, level) = match self.data.as_ref() {
            Some(data) => data,
            None => return,
        };

        let border_color = &self.cfg.colors.focus_border;
        let (border_style, border_type) = super::get_border_style(border_color, border_color, true);

        let (title, text_style) = match level {
            PopupLevel::Error => ("error", self.cfg.colors.popup.error_text.style),
        };

        let block = Block::new()
            .border_type(border_type)
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_alignment(Alignment::Center)
            .title(title);

        let text = Text::from(text.as_str());

        let widget = Paragraph::new(text)
            .style(text_style)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll as u16, 0));

        let area = Self::centered_rect(50, 50, frame.size());
        frame.render_widget(Clear, area); // this clears out the background
        frame.render_widget(widget, area);
    }

    /// helper function to create a centered rect using up certain percentage of the
    /// available rect `r`
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::vertical([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

        Layout::horizontal([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
    }
}
