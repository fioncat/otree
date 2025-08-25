use std::rc::Rc;

use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::config::Config;

pub enum FooterText<'a> {
    Identify(&'a [String], Option<String>),
    Message(String),
    None,
}

pub struct Footer {
    cfg: Rc<Config>,
}

impl Footer {
    pub fn new(cfg: Rc<Config>) -> Self {
        Self { cfg }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, text: FooterText) {
        let line = match text {
            FooterText::Identify(roots, identify) => {
                let mut spans = Vec::with_capacity(roots.len() * 2 + 1);
                let mut widths = vec![];
                for root in roots {
                    let root = format!(" /{root} ");
                    widths.push(root.len() + 1);
                    spans.push(Span::styled(root, self.cfg.colors.footer.root.style));
                    spans.push(Span::raw(" "));
                }

                let mut identify_width = 0;
                let identify_span = identify.map(|id| {
                    let identify = format!(" /{id} ");
                    identify_width = identify.len();
                    Span::styled(identify, self.cfg.colors.footer.identify.style)
                });

                if let Some(span) = identify_span.clone() {
                    spans.push(span);
                    widths.push(identify_width);
                }

                for (idx, root) in roots.iter().enumerate() {
                    let total_width: usize = widths.iter().sum();
                    if total_width < area.width as usize {
                        break;
                    }
                    let first_ch = root.chars().next().unwrap_or('?');
                    let short_root = format!(" /{first_ch}.. ");
                    widths[idx] = short_root.len() + 1;
                    spans[idx * 2] = Span::styled(short_root, self.cfg.colors.footer.root.style);
                }

                if widths.iter().sum::<usize>() > area.width as usize {
                    // Reserve space for the count of omitted roots. (N.. )
                    let reserve_width = roots.len().to_string().len() + 3;
                    widths.push(reserve_width);
                    let mut omit_count = 0;
                    for idx in 0..roots.len() {
                        let mut total_width: usize = widths.iter().sum();
                        total_width += reserve_width;
                        if total_width < area.width as usize {
                            break;
                        }
                        widths[idx] = 0;
                        omit_count += 1;
                    }

                    if omit_count > 0 {
                        for _ in 0..omit_count {
                            spans.remove(0);
                            spans.remove(0);
                        }
                        let omit_span = Span::styled(
                            format!("{omit_count}.."),
                            self.cfg.colors.footer.root.style,
                        );
                        spans.insert(0, omit_span);
                        spans.insert(1, Span::raw(" "));
                    }
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
