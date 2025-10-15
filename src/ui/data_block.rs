use std::rc::Rc;

use ratatui::layout::{Alignment, Margin, Rect};
use ratatui::symbols::scrollbar;
use ratatui::text::Text;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::Frame;

use crate::config::keys::Action;
use crate::config::Config;
use crate::tree::ItemValue;
use crate::ui::app::ScrollDirection;

pub struct DataBlock {
    cfg: Rc<Config>,

    can_vertical_scroll: bool,
    vertical_scroll: usize,
    vertical_scroll_last: usize,
    vertical_scroll_state: ScrollbarState,

    can_horizontal_scroll: bool,
    horizontal_scroll: usize,
    horizontal_scroll_last: usize,
    horizontal_scroll_state: ScrollbarState,

    identify: String,
}

impl DataBlock {
    const SCROLL_RETAIN: usize = 5;

    pub fn new(cfg: Rc<Config>) -> Self {
        Self {
            cfg,
            can_vertical_scroll: false,
            vertical_scroll: 0,
            vertical_scroll_last: 0,
            vertical_scroll_state: ScrollbarState::default(),
            can_horizontal_scroll: false,
            horizontal_scroll: 0,
            horizontal_scroll_last: 0,
            horizontal_scroll_state: ScrollbarState::default(),
            identify: String::default(),
        }
    }

    pub fn on_key(&mut self, action: Action) -> bool {
        match action {
            Action::MoveDown => self.scroll_down(1),
            Action::MoveUp => self.scroll_up(1),
            Action::MoveRight => self.scroll_right(1),
            Action::MoveLeft => self.scroll_left(1),
            Action::SelectFirst => self.scroll_first(),
            Action::SelectLast => self.scroll_last(),
            _ => false,
        }
    }

    pub fn on_scroll(&mut self, direction: ScrollDirection) -> bool {
        match direction {
            ScrollDirection::Up => self.scroll_up(3),
            ScrollDirection::Down => self.scroll_down(3),
        }
    }

    pub fn scroll_first(&mut self) -> bool {
        let can_scroll = self.can_vertical_scroll || self.can_horizontal_scroll;
        let scroll_first = self.vertical_scroll == 0 && self.horizontal_scroll == 0;

        if !can_scroll || scroll_first {
            return false;
        }

        self.vertical_scroll = 0;
        self.vertical_scroll_state = self.vertical_scroll_state.position(0);

        self.horizontal_scroll = 0;
        self.horizontal_scroll_state = self.horizontal_scroll_state.position(0);

        true
    }

    pub fn scroll_last(&mut self) -> bool {
        if !self.can_vertical_scroll || self.vertical_scroll == self.vertical_scroll_last {
            return false;
        }

        self.vertical_scroll = self.vertical_scroll_last;
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .position(self.vertical_scroll_last);

        true
    }

    pub fn scroll_down(&mut self, lines: usize) -> bool {
        if !self.can_vertical_scroll || self.vertical_scroll == self.vertical_scroll_last {
            return false;
        }

        self.vertical_scroll = self.vertical_scroll.saturating_add(lines);
        if self.vertical_scroll > self.vertical_scroll_last {
            self.vertical_scroll = self.vertical_scroll_last;
        }
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
        true
    }

    pub fn scroll_up(&mut self, lines: usize) -> bool {
        if !self.can_vertical_scroll || self.vertical_scroll == 0 {
            return false;
        }

        self.vertical_scroll = self.vertical_scroll.saturating_sub(lines);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
        true
    }

    pub fn scroll_right(&mut self, lines: usize) -> bool {
        if !self.can_horizontal_scroll || self.horizontal_scroll == self.horizontal_scroll_last {
            return false;
        }
        self.horizontal_scroll = self.horizontal_scroll.saturating_add(lines);
        if self.horizontal_scroll > self.horizontal_scroll_last {
            self.horizontal_scroll = self.horizontal_scroll_last;
        }
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
        true
    }

    pub fn scroll_left(&mut self, lines: usize) -> bool {
        if self.horizontal_scroll == 0 {
            return false;
        }
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(lines);
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
        true
    }

    pub fn draw(
        &mut self,
        item: Option<(String, &ItemValue)>,
        frame: &mut Frame,
        area: Rect,
        focus: bool,
    ) {
        let text = match item {
            Some((identify, item)) => {
                let (text, rows, cols) = item.render(self.cfg.as_ref(), area.width as usize);
                self.update_scroll(identify, rows, cols, area);
                text
            }
            None => {
                self.reset();
                Text::default()
            }
        };
        let (border_style, border_type) = super::get_border_style(
            &self.cfg.colors.focus_border,
            &self.cfg.colors.data.border,
            focus,
        );

        let block = Block::new()
            .border_type(border_type)
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_alignment(Alignment::Center)
            .title("Data Block");

        let widget = Paragraph::new(text)
            .style(self.cfg.colors.data.text.style)
            .block(block)
            // TODO: figure out if this cast actually causes problems
            // example: json array with 2^16 entries
            .scroll((
                u16::try_from(self.vertical_scroll).unwrap_or_default(),
                u16::try_from(self.horizontal_scroll).unwrap_or_default(),
            ));

        frame.render_widget(widget, area);

        if self.can_vertical_scroll {
            let vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None);

            frame.render_stateful_widget(
                vertical_scrollbar,
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut self.vertical_scroll_state,
            );
        }

        if self.can_horizontal_scroll {
            let horizontal_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .symbols(scrollbar::HORIZONTAL)
                .begin_symbol(None)
                .thumb_symbol("🬋")
                .track_symbol(None)
                .end_symbol(None);

            frame.render_stateful_widget(
                horizontal_scrollbar,
                area.inner(Margin {
                    vertical: 0,
                    horizontal: 1,
                }),
                &mut self.horizontal_scroll_state,
            );
        }
    }

    fn update_scroll(&mut self, identify: String, rows: usize, cols: usize, area: Rect) {
        if self.identify != identify {
            self.reset_scroll();
            self.identify = identify;
        }

        let rows = rows + Self::SCROLL_RETAIN;
        if rows > area.height as usize {
            self.can_vertical_scroll = true;
            self.vertical_scroll_last = rows.saturating_sub(area.height as usize);
            self.vertical_scroll_state = self
                .vertical_scroll_state
                .content_length(self.vertical_scroll_last);
        } else {
            self.reset_scroll_vertical();
        }

        let columns = cols + Self::SCROLL_RETAIN;
        if !self.cfg.data.wrap && columns > area.width as usize {
            self.can_horizontal_scroll = true;
            self.horizontal_scroll_last = columns.saturating_sub(area.width as usize);
            self.horizontal_scroll_state = self
                .horizontal_scroll_state
                .content_length(self.horizontal_scroll_last);
        } else {
            self.reset_scroll_horizontal();
        }
    }

    fn reset(&mut self) {
        self.reset_scroll();
        self.identify.clear();
    }

    fn reset_scroll(&mut self) {
        self.reset_scroll_vertical();
        self.reset_scroll_horizontal();
    }

    fn reset_scroll_vertical(&mut self) {
        // Reset all vertical scroll state.
        self.can_vertical_scroll = false;
        self.vertical_scroll_state = ScrollbarState::default();
        self.vertical_scroll = 0;
        self.vertical_scroll_last = 0;
    }

    fn reset_scroll_horizontal(&mut self) {
        // Reset all horizontal scroll state.
        self.can_horizontal_scroll = false;
        self.horizontal_scroll_state = ScrollbarState::default();
        self.horizontal_scroll = 0;
        self.horizontal_scroll_last = 0;
    }
}
