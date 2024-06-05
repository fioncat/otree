use ratatui::text::{Line, Span, Text};

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
        let mut lines: Vec<Line> = vec![];
        let mut current_line = Some(Line::default());
        for token in tokens {
            let (token, style) = match token {
                Self::Symbol(sym) => (*sym, cfg.colors.data.symbol.style),
                Self::Name(name) => (name.as_str(), cfg.colors.data.name.style),
                Self::String(str) => (str.as_str(), cfg.colors.data.str.style),
                Self::Number(num) => (num.as_str(), cfg.colors.data.num.style),
                Self::Null(null) => (*null, cfg.colors.data.null.style),
                Self::Bool(b) => (*b, cfg.colors.data.bool.style),
                Self::Section(sec) => (sec.as_str(), cfg.colors.data.section.style),
                Self::Break => {
                    let line = current_line.take().unwrap();
                    lines.push(line);
                    current_line = Some(Line::default());
                    continue;
                }
                Self::Indent(indent) => {
                    let indent = "  ".repeat(*indent);
                    current_line.as_mut().unwrap().push_span(Span::raw(indent));
                    continue;
                }
            };
            current_line
                .as_mut()
                .unwrap()
                .push_span(Span::styled(token, style));
        }
        if current_line.as_ref().unwrap().width() > 0 {
            let line = current_line.take().unwrap();
            lines.push(line);
        }

        Text::from(lines)
    }

    pub fn get_size(tokens: &[SyntaxToken]) -> (usize, usize) {
        let mut rows: usize = 0;
        let mut current_columns: usize = 0;
        let mut max_columns: usize = 0;
        for token in tokens {
            match token {
                Self::Symbol(sym) => current_columns += sym.len(),
                Self::Name(name) => current_columns += name.len(),
                Self::String(str) => current_columns += str.len(),
                Self::Number(num) => current_columns += num.len(),
                Self::Null(null) => current_columns += null.len(),
                Self::Bool(b) => current_columns += b.len(),
                Self::Section(sec) => current_columns += sec.len(),
                Self::Break => {
                    if current_columns > max_columns {
                        max_columns = current_columns;
                    }
                    current_columns = 0;
                    rows += 1;
                }
                Self::Indent(indent) => current_columns += 2 * indent,
            }
        }

        if current_columns > 0 {
            if current_columns > max_columns {
                max_columns = current_columns;
            }
            rows += 1;
        }

        (rows, max_columns)
    }

    #[cfg(test)]
    pub(super) fn pure_text(tokens: &[SyntaxToken]) -> String {
        let mut text = String::new();
        for token in tokens {
            let token = match token {
                Self::Symbol(sym) => sym,
                Self::Name(name) => name.as_str(),
                Self::String(str) => str.as_str(),
                Self::Number(num) => num.as_str(),
                Self::Null(null) => null,
                Self::Bool(b) => b,
                Self::Section(sec) => sec.as_str(),
                Self::Break => "\n",
                Self::Indent(indent) => {
                    for _ in 0..*indent {
                        text.push_str("  ");
                    }
                    continue;
                }
            };
            text.push_str(token);
        }
        text
    }
}
