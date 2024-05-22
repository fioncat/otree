use std::borrow::Cow;

use ratatui::layout::{Alignment, Rect};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::config::Config;
use crate::tree::ContentType;

pub struct HeaderContext {
    version: String,
    data_source: Cow<'static, str>,
    content_type: &'static str,
    data_size: String,
}

impl HeaderContext {
    pub fn new(source: Option<String>, content_type: ContentType, size: usize) -> Self {
        let version = format!("otree {}", env!("CARGO_PKG_VERSION"));
        let source = source.map(Cow::Owned).unwrap_or(Cow::Borrowed("stdin"));
        let content_type = match content_type {
            ContentType::Toml => "toml",
            ContentType::Yaml => "yaml",
            ContentType::Json => "json",
        };

        let data_size = humansize::format_size(size, humansize::BINARY);

        Self {
            version,
            data_source: source,
            content_type,
            data_size,
        }
    }

    fn format(&self, s: &str) -> String {
        let s = s.replace("{version}", &self.version);
        let s = s.replace("{data_source}", &self.data_source);
        let s = s.replace("{content_type}", self.content_type);
        s.replace("{data_size}", &self.data_size)
    }
}

pub(super) struct Header<'a> {
    cfg: &'a Config,
    data: String,
}

impl<'a> Header<'a> {
    pub(super) fn new(cfg: &'a Config, ctx: HeaderContext) -> Self {
        Self {
            cfg,
            data: ctx.format(&cfg.header.format),
        }
    }

    pub(super) fn draw(&self, frame: &mut Frame, area: Rect) {
        let span = Span::styled(self.data.as_str(), self.cfg.colors.header.style);
        // TODO: Allow user to customize alignment.
        let paragraph = Paragraph::new(span).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
}
