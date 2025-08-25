use std::borrow::Cow;
use std::rc::Rc;

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::config::keys::Action;
use crate::config::Config;
use crate::ui::app::ScrollDirection;

pub struct PopupData {
    pub title: Cow<'static, str>,
    pub text: Text<'static>,
}

pub struct Popup {
    data: Option<PopupData>,

    cfg: Rc<Config>,

    scroll: usize,
}

impl Popup {
    pub fn new(cfg: Rc<Config>) -> Self {
        Self {
            data: None,
            cfg,
            scroll: 0,
        }
    }

    pub fn set_data(&mut self, data: PopupData) {
        self.data = Some(data);
    }

    pub fn on_key(&mut self, action: Action) -> bool {
        match action {
            Action::MoveDown => self.scroll_down(1),
            Action::MoveUp => self.scroll_up(1),
            Action::SelectFocus | Action::Reset => self.disable(),
            _ => false,
        }
    }

    pub fn on_scroll(&mut self, direction: ScrollDirection) -> bool {
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

    pub fn disable(&mut self) -> bool {
        if self.data.is_none() {
            return false;
        }

        self.data = None;
        self.scroll = 0;
        true
    }

    pub fn is_disabled(&self) -> bool {
        self.data.is_none()
    }

    pub fn draw(&self, frame: &mut Frame) {
        let Some(data) = self.data.as_ref() else {
            return;
        };

        let border_color = &self.cfg.colors.focus_border;
        let (border_style, border_type) = super::get_border_style(border_color, border_color, true);

        let block = Block::new()
            .border_type(border_type)
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_alignment(Alignment::Center)
            .title(data.title.as_ref());

        let widget = Paragraph::new(data.text.clone())
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((u16::try_from(self.scroll).unwrap_or_default(), 0));

        let area = Self::centered_rect(50, 50, frame.area());
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
