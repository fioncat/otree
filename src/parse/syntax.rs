use std::borrow::Cow;
use std::sync::LazyLock;

use ratatui::style::Style;
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
    const WRAP_WIDTH_RESERVED: usize = 3;

    pub fn render<'a>(
        cfg: &Config,
        tokens: &'a [SyntaxToken],
        width: usize,
    ) -> (Text<'a>, usize, usize) {
        let width = width.saturating_sub(Self::WRAP_WIDTH_RESERVED);

        let mut lines: Vec<Line> = vec![];
        let mut current_line = Some(Line::default());
        let mut current_width = 0;
        let mut rows = 0;
        let mut max_width = 0;
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
                    if current_width > max_width {
                        max_width = current_width;
                    }
                    current_width = 0;
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

            if cfg.data.wrap {
                let wrapper = TextWrapper {
                    lines: &mut lines,
                    current_line: &mut current_line,
                    current_width: &mut current_width,
                    text: token,
                    style,
                    max_area_width: width,
                    rows: &mut rows,
                    max_text_width: &mut max_width,
                };
                wrapper.add_text_with_wrap(cfg);
                continue;
            }

            current_width += console::measure_text_width(token.as_ref());
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

        (Text::from(lines), rows, max_width)
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

struct TextWrapper<'a, 'this> {
    lines: &'this mut Vec<Line<'a>>,
    current_line: &'this mut Option<Line<'a>>,
    current_width: &'this mut usize,
    text: Cow<'a, str>,
    style: Option<Style>,
    max_area_width: usize,

    rows: &'this mut usize,
    max_text_width: &'this mut usize,
}

impl TextWrapper<'_, '_> {
    const WRAP_SYMBOL: &'static str = "⤷ ";
    const WRAP_SYMBOL_LEN: usize = Self::WRAP_SYMBOL.len();

    const WRAP_RESERVED: usize = 5;

    fn add_text_with_wrap(mut self, cfg: &Config) {
        if self.text.is_empty() {
            return;
        }
        if self.max_area_width == 0 {
            return;
        }

        while !self.text.is_empty() {
            let available_width = if *self.current_width == 0 {
                self.max_area_width
            } else {
                self.max_area_width.saturating_sub(*self.current_width)
            };

            if available_width == 0 {
                // Need to wrap to a new line
                self.wrap_new_line(cfg);
                continue;
            }

            let remaining_width = console::measure_text_width(&self.text);
            if remaining_width <= available_width {
                // Entire remaining text fits in the available width
                let span = if let Some(style) = self.style {
                    Span::styled(self.text, style)
                } else {
                    Span::raw(self.text)
                };
                self.current_line.as_mut().unwrap().push_span(span);
                *self.current_width += remaining_width;
                break;
            }

            // Need to split the text
            let split_pos = find_split_position(&self.text, available_width);
            let (current_part, rest) = self.text.split_at(split_pos);
            let current_part_width = console::measure_text_width(current_part);

            let span = if let Some(style) = self.style {
                Span::styled(current_part.to_string(), style)
            } else {
                Span::raw(current_part.to_string())
            };
            self.current_line.as_mut().unwrap().push_span(span);
            *self.current_width += current_part_width;

            // Prepare for the next iteration
            self.text = Cow::Owned(rest.to_string());

            if !self.text.is_empty() {
                self.wrap_new_line(cfg);
            }
        }
    }

    fn wrap_new_line(&mut self, cfg: &Config) {
        let line = self.current_line.take().unwrap();
        self.lines.push(line);
        *self.current_line = Some(Line::default());
        *self.rows += 1;
        if *self.current_width > *self.max_text_width {
            *self.max_text_width = *self.current_width;
        }
        if Self::WRAP_SYMBOL_LEN + Self::WRAP_RESERVED < self.max_area_width {
            self.current_line.as_mut().unwrap().push_span(Span::styled(
                Self::WRAP_SYMBOL,
                cfg.colors.data.symbol.style,
            ));
            *self.current_width = Self::WRAP_SYMBOL_LEN;
        } else {
            *self.current_width = 0;
        }
    }
}

fn find_split_position(text: &str, max_width: usize) -> usize {
    if text.len() <= max_width {
        return text.len();
    }

    // Try to find a good break point (space and table)
    let bytes = text.as_bytes();
    let mut best_pos = max_width;

    // Look backwards from max_width to find a good break point
    for i in (max_width.saturating_sub(10)..max_width).rev() {
        if i >= bytes.len() {
            continue;
        }

        match bytes[i] {
            b' ' | b'\t' => {
                best_pos = i + 1; // Include the separator in the current part
                break;
            }
            _ => {}
        }
    }

    // Ensure we don't split in the middle of a UTF-8 character
    while best_pos > 0 && !text.is_char_boundary(best_pos) {
        best_pos -= 1;
    }

    // Ensure we make progress (don't return 0 unless the text is empty)
    if best_pos == 0 && !text.is_empty() {
        // Find the next character boundary after position 1
        best_pos = 1;
        while best_pos < text.len() && !text.is_char_boundary(best_pos) {
            best_pos += 1;
        }
    }

    best_pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_split_position() {
        let cases = [
            ("short text", 20, 10),
            ("", 10, 0),
            ("a", 10, 1),
            ("hello", 5, 5),
            ("hi", 10, 2),
            ("hello world", 8, 6),
            ("verylongword", 5, 5),
            ("测试中文", 4, 3),
            ("🚀rocket", 4, 4),
            ("café", 3, 3),
            ("naïve", 3, 2),
        ];

        for (text, max_width, expected) in cases {
            let pos = find_split_position(text, max_width);
            assert_eq!(pos, expected, "Failed for text: {text:?}");
        }
    }

    #[test]
    fn test_split() {
        let cases = [
            ("short text", 20, vec!["short text"]),
            ("", 10, vec![]),
            ("a", 10, vec!["a"]),
            ("hello", 5, vec!["hello"]),
            ("hi", 10, vec!["hi"]),
            ("hello world", 8, vec!["hello ", "world"]),
            ("verylongword", 5, vec!["veryl", "ongwo", "rd"]),
            ("测试中文", 7, vec!["测试", "中文"]),
            ("🚀rocket", 4, vec!["🚀", "rock", "et"]),
            ("café", 3, vec!["caf", "é"]),
            ("naïve", 3, vec!["na", "ïv", "e"]),
            ("abcdefghijklmnop", 5, vec!["abcde", "fghij", "klmno", "p"]),
            ("test with spaces", 6, vec!["test ", "with ", "spaces"]),
            (
                "test with spaces",
                3,
                vec!["tes", "t ", "wit", "h ", "spa", "ces"],
            ),
        ];

        for (text, max_width, expected) in cases {
            let mut parts = Vec::new();
            let mut remaining = text;

            while !remaining.is_empty() {
                let pos = find_split_position(remaining, max_width);
                let (current, rest) = remaining.split_at(pos);
                parts.push(current);
                remaining = rest;
            }

            assert_eq!(
                parts, expected,
                "Split failed for text: {text:?}, max_width: {max_width}"
            );

            let rejoined = parts.join("");
            assert_eq!(
                rejoined, text,
                "Rejoined text doesn't match original for: {text:?}"
            );

            for part in &parts {
                assert!(
                    part.len() <= max_width || part.chars().count() == 1,
                    "Part too long: {part:?} (len: {}, max: {max_width})",
                    part.len()
                );
            }
        }
    }
}
