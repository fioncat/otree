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

    #[cfg(test)]
    pub(super) fn pure_text(tokens: &[SyntaxToken]) -> String {
        let mut text = String::new();
        for token in tokens {
            if let Self::Indent(indent) = token {
                for _ in 0..*indent {
                    text.push_str("  ");
                }
                continue;
            }

            let token = match token {
                Self::Symbol(sym) => sym,
                Self::Name(name) => name.as_str(),
                Self::String(str) => str.as_str(),
                Self::Number(num) => num.as_str(),
                Self::Null(null) => null,
                Self::Bool(b) => b,
                Self::Section(sec) => sec.as_str(),
                Self::Break => "\n",
                _ => unreachable!(),
            };
            text.push_str(token);
        }
        text
    }
}
