use std::rc::Rc;

use ansi_to_tui::IntoText;
use once_cell::sync::Lazy;
use ratatui::text::Text;
use serde_json::Value;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

use crate::config::Config;
use crate::parse::Parser;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

pub struct SyntaxText {
    pub text: Text<'static>,
    pub columns: usize,
    pub rows: usize,
}

impl SyntaxText {
    pub fn parse(cfg: &Config, value: &Value, parser: Rc<Box<dyn Parser>>) -> Self {
        if cfg.data.disable_highlight {
            let value = parser.to_string(value);
            return Self::pure(value);
        }

        match value {
            Value::Null => return Self::pure(String::new()),
            Value::String(s) => return Self::pure(s.to_string()),
            Value::Number(num) => return Self::pure(num.to_string()),
            Value::Bool(b) => return Self::pure(b.to_string()),
            _ => {}
        }

        let value = parser.to_string(value);
        Self::highlighting(value, parser.extension())
    }

    fn pure(value: String) -> Self {
        let (columns, rows) = Self::get_size(&value);
        Self {
            text: Text::from(value),
            columns,
            rows,
        }
    }

    fn highlighting(value: String, extension: &str) -> Self {
        let syntax = match SYNTAX_SET.find_syntax_by_extension(extension) {
            Some(syntax) => syntax,
            // TODO: When cannot find syntax by extension, we should warn user
            None => return Self::pure(value),
        };

        let mut h = HighlightLines::new(syntax, &THEME_SET.themes["base16-ocean.dark"]);
        let hightlighted = LinesWithEndings::from(&value)
            .map(|line| {
                let ranges: Vec<(syntect::highlighting::Style, &str)> =
                    h.highlight_line(line, &SYNTAX_SET).unwrap();
                as_24_bit_terminal_escaped(&ranges[..], false)
            })
            .collect::<Vec<String>>()
            .join("");

        let (columns, rows) = Self::get_size(&value);
        let text = hightlighted.into_text().unwrap();

        Self {
            text,
            columns,
            rows,
        }
    }

    fn get_size(s: &str) -> (usize, usize) {
        let lines: Vec<&str> = s.lines().collect();
        let columns = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let rows = lines.len();
        (columns, rows)
    }
}
