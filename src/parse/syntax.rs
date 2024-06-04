use ratatui::text::Text;

use crate::config::Config;

pub enum SyntaxToken {
    Symbol(&'static str),

    Name(String),

    String(String),
    Number(String),
    Null(&'static str),
    Bool(&'static str),

    Section(String),

    Break,
    Indent(usize),
}

impl SyntaxToken {
    pub fn render<'a>(cfg: &Config, tokens: &'a [SyntaxToken]) -> Text<'a> {
        todo!()
    }

    pub fn get_size(tokens: &[SyntaxToken]) -> (usize, usize) {
        todo!()
    }
}
