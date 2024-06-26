use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::config::Config;

pub(super) enum FooterText<'a> {
    Identify(&'a [String], Option<String>),
    Message(String),
    None,
}

pub(super) struct Footer<'a> {
    cfg: &'a Config,
}

impl<'a> Footer<'a> {
    pub(super) fn new(cfg: &'a Config) -> Self {
        Self { cfg }
    }

    pub(super) fn draw(&self, frame: &mut Frame, area: Rect, text: FooterText) {
        let line = match text {
            FooterText::Identify(roots, identify) => {
                let mut spans = Vec::with_capacity(roots.len() * 2 + 1);
                for root in roots {
                    let root = format!(" /{root} ");
                    spans.push(Span::styled(root, self.cfg.colors.footer.root.style));
                    spans.push(Span::raw(" "));
                }

                if let Some(identify) = identify {
                    let identify = format!(" /{identify} ");
                    spans.push(Span::styled(
                        identify,
                        self.cfg.colors.footer.identify.style,
                    ));
                }

                Line::from(spans)
            }
            FooterText::Message(msg) => Line::styled(msg, self.cfg.colors.footer.message.style),
            FooterText::None => return,
        };
        // TODO: Allow user to customize alignment.
        let paragraph = Paragraph::new(line).alignment(Alignment::Left);
        frame.render_widget(paragraph, area);
    }
}
