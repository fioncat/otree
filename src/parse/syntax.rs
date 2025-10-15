use std::borrow::Cow;
use std::sync::LazyLock;

use ratatui::text::{Line, Span, Text};
use regex::Regex;
use serde_json::Value;

use crate::config::Config;

pub enum SyntaxToken {
    Symbol(&'static str),

    Name(String),

    Tag(String),

    String(String),
    Number(String),
    Null(&'static str),
    Bool(&'static str),

    Section(String),

    Break,
    Indent(usize),
}

impl SyntaxToken {
    const WRAP_SYMBOL: &str = "⤷ ";
    pub const WRAP_SYMBOL_LEN: usize = Self::WRAP_SYMBOL.len();

    pub fn render<'a>(
        cfg: &Config,
        tokens: &'a [SyntaxToken],
        width: usize,
    ) -> (Text<'a>, usize, usize) {
        let mut lines: Vec<Line> = vec![];
        let mut current_line = Some(Line::default());
        let mut rows = 0;
        let mut cols = 0;
        let mut max_cols = 0;
        for token in tokens {
            let (token, mut style) = match token {
                Self::Symbol(sym) => (Cow::Borrowed(*sym), Some(cfg.colors.data.symbol.style)),
                Self::Name(name) => (
                    Cow::Borrowed(name.as_str()),
                    Some(cfg.colors.data.name.style),
                ),
                Self::Tag(tag) => (Cow::Borrowed(tag.as_str()), Some(cfg.colors.data.tag.style)),
                Self::String(str) => (Cow::Borrowed(str.as_str()), Some(cfg.colors.data.str.style)),
                Self::Number(num) => (Cow::Borrowed(num.as_str()), Some(cfg.colors.data.num.style)),
                Self::Null(null) => (Cow::Borrowed(*null), Some(cfg.colors.data.null.style)),
                Self::Bool(b) => (Cow::Borrowed(*b), Some(cfg.colors.data.bool.style)),
                Self::Section(sec) => (
                    Cow::Borrowed(sec.as_str()),
                    Some(cfg.colors.data.section.style),
                ),
                Self::Break => {
                    let line = current_line.take().unwrap();
                    lines.push(line);
                    current_line = Some(Line::default());
                    rows += 1;
                    if cols > max_cols {
                        max_cols = cols;
                    }
                    cols = 0;
                    continue;
                }
                Self::Indent(indent) => {
                    let indent = "  ".repeat(*indent);
                    (indent.into(), None)
                }
            };
            if cfg.data.disable_highlight {
                style = None;
            }
            cols += console::measure_text_width(token.as_ref());
            let current_line = current_line.as_mut().unwrap();
            match style {
                Some(style) => current_line.push_span(Span::styled(token, style)),
                None => current_line.push_span(Span::raw(token)),
            }
        }
        if current_line.as_ref().unwrap().width() > 0 {
            let line = current_line.take().unwrap();
            lines.push(line);
        }

        (Text::from(lines), rows, max_cols)
    }

    pub fn pure_text(tokens: &[SyntaxToken]) -> String {
        let mut text = String::new();
        for token in tokens {
            let token = match token {
                Self::Symbol(sym) => sym,
                Self::Name(name) => name.as_str(),
                Self::Tag(tag) => tag.as_str(),
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

static STANDARD_FIELD_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());

pub fn quote_field_name(name: &str) -> String {
    if STANDARD_FIELD_NAME_RE.is_match(name) {
        // This is a standard field name, we don't need to quote it.
        // Like: "version", "dev-dependencies"
        name.to_string()
    } else {
        // Not a standard field name, quote it.
        // Like: "/some/path", "with space" (include empty string "")
        format!("{name:?}")
    }
}

pub enum StringValue {
    String(String),
    MultiLines(Vec<String>),
}

impl StringValue {
    pub fn new(s: &str, must_quote: bool) -> Self {
        if s.is_empty() {
            return StringValue::String("\"\"".to_string());
        }

        let trim_str = s.trim();
        if !trim_str.is_empty() && s.contains('\n') {
            // If the string contains newlines, we should use the multiline string feature
            // provided by the schema language. This feature is available in both YAML and
            // TOML. Therefore, a unified method for judgment is defined here.
            // A special case is strings that only contain special characters, such as
            // "\n\t\n\n". These are not considered normal multiline strings, so they should
            // follow the logic below for quoting.
            let lines: Vec<_> = s.lines().map(str::to_string).collect();
            return StringValue::MultiLines(lines);
        }

        if must_quote {
            return StringValue::String(Self::quote_string(s));
        }

        if s == "true" || s == "false" || Self::is_numeric(s) {
            // Special strings. Boolean or numeric. These strings should be quoted to prevent
            // the schema parser from interpreting them as actual booleans or numbers, as
            // they are actually strings.
            return StringValue::String(Self::quote_string(s));
        }

        if STANDARD_FIELD_NAME_RE.is_match(s) {
            StringValue::String(s.to_string())
        } else {
            StringValue::String(Self::quote_string(s))
        }
    }

    fn quote_string(s: &str) -> String {
        format!("{s:?}")
    }

    fn is_numeric(s: &str) -> bool {
        s.parse::<f64>().is_ok()
    }
}

pub fn is_value_complex(value: &Value) -> bool {
    match value {
        Value::Object(_) => true,
        Value::Array(arr) => {
            for value in arr {
                if is_value_complex(value) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}
