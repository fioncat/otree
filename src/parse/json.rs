use anyhow::{Context, Result};
use serde_json::Value;

use super::{Parser, SyntaxToken};

pub struct JsonParser {}

impl Parser for JsonParser {
    fn extension(&self) -> &'static str {
        "json"
    }

    fn parse(&self, data: &str) -> Result<Value> {
        serde_json::from_str(data).context("parse JSON")
    }

    fn to_string(&self, value: &Value) -> String {
        serde_json::to_string_pretty(value).expect("serialize JSON")
    }

    fn syntax_highlight(&self, value: &Value) -> Vec<SyntaxToken> {
        highlight(value, 0, false)
    }
}

pub fn highlight(value: &Value, indent: usize, has_next: bool) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    match value {
        Value::Null => tokens.push(SyntaxToken::Null("null")),
        Value::String(s) => tokens.push(SyntaxToken::String(format!("{s:?}"))),
        Value::Number(num) => tokens.push(SyntaxToken::Number(num.to_string())),
        Value::Bool(b) => {
            let b = if *b { "true" } else { "false" };
            tokens.push(SyntaxToken::Bool(b));
        }
        Value::Object(obj) => {
            tokens.push(SyntaxToken::Symbol("{"));
            if obj.is_empty() {
                tokens.push(SyntaxToken::Symbol("}"));
            } else {
                tokens.push(SyntaxToken::Break);
                for (idx, (field, value)) in obj.iter().enumerate() {
                    tokens.push(SyntaxToken::Indent(indent + 1));
                    let field = format!("{field:?}");
                    tokens.push(SyntaxToken::Name(field));
                    tokens.push(SyntaxToken::Symbol(": "));

                    let has_next = idx != obj.len() - 1;
                    let value_tokens = highlight(value, indent + 1, has_next);
                    tokens.extend(value_tokens);
                }
                tokens.push(SyntaxToken::Indent(indent));
                tokens.push(SyntaxToken::Symbol("}"));
            }
        }
        Value::Array(arr) => {
            tokens.push(SyntaxToken::Symbol("["));
            if arr.is_empty() {
                tokens.push(SyntaxToken::Symbol("]"));
            } else {
                tokens.push(SyntaxToken::Break);
                for (idx, value) in arr.iter().enumerate() {
                    tokens.push(SyntaxToken::Indent(indent + 1));
                    let has_next = idx != arr.len() - 1;
                    let value_tokens = highlight(value, indent + 1, has_next);
                    tokens.extend(value_tokens);
                }
                tokens.push(SyntaxToken::Indent(indent));
                tokens.push(SyntaxToken::Symbol("]"));
            }
        }
    }

    if has_next {
        tokens.push(SyntaxToken::Symbol(","));
    }
    tokens.push(SyntaxToken::Break);

    tokens
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_syntax_highlight() {
        let test_cases = [
            (
                include_str!("test_cases/json/object.json"),
                include_str!("test_cases/json/object_highlight.json"),
            ),
            (
                include_str!("test_cases/json/array_of_objects.json"),
                include_str!("test_cases/json/array_of_objects_highlight.json"),
            ),
            (
                include_str!("test_cases/json/2d_array.json"),
                include_str!("test_cases/json/2d_array.json"),
            ),
            (
                include_str!("test_cases/json/3d_array.json"),
                include_str!("test_cases/json/3d_array.json"),
            ),
            (
                include_str!("test_cases/json/empty.json"),
                include_str!("test_cases/json/empty.json"),
            ),
        ];

        let parser = JsonParser {};
        for (raw, expect) in test_cases {
            let value = parser.parse(raw).unwrap();
            let tokens = parser.syntax_highlight(&value);
            let result = SyntaxToken::pure_text(&tokens);
            assert_eq!(result, expect);

            let highlight_value = parser.parse(&result).unwrap();
            assert_eq!(value, highlight_value);
        }
    }
}
