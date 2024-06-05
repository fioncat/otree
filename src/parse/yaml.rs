use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::{Parser, SyntaxToken};

pub(super) struct YamlParser {}

impl Parser for YamlParser {
    fn parse(&self, data: &str) -> Result<Value> {
        let mut values = Vec::with_capacity(1);
        for document in serde_yml::Deserializer::from_str(data) {
            let value = Value::deserialize(document).context("parse YAML")?;
            values.push(value);
        }

        if values.is_empty() {
            bail!("no document found in YAML data");
        }

        if values.len() == 1 {
            return Ok(values.into_iter().next().unwrap());
        }

        Ok(Value::Array(values))
    }

    fn to_string(&self, value: &Value) -> String {
        serde_yml::to_string(value).expect("serialize YAML")
    }

    fn syntax_highlight(&self, value: &Value) -> Vec<SyntaxToken> {
        let mut tokens = if let Value::Array(arr) = value {
            // YAML Multi Documents
            // See: <https://gettaurus.org/docs/YAMLTutorial/#YAML-Multi-Documents>
            let mut tokens = Vec::new();
            for (idx, value) in arr.iter().enumerate() {
                let document_tokens = highlight(value, 0, false);
                tokens.extend(document_tokens);

                if idx != arr.len() - 1 {
                    tokens.push(SyntaxToken::Break);
                    tokens.push(SyntaxToken::Symbol("---"));
                    tokens.push(SyntaxToken::Break);
                }
            }
            tokens
        } else {
            highlight(value, 0, false)
        };

        // Pop the last break
        tokens.pop();
        tokens
    }
}

fn highlight(value: &Value, indent: usize, from_arr: bool) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    match value {
        Value::Null => tokens.push(SyntaxToken::Null("null")),
        Value::String(s) => append_string_tokens(&mut tokens, s, indent),
        Value::Number(num) => tokens.push(SyntaxToken::Number(num.to_string())),
        Value::Bool(b) => {
            let b = if *b { "true" } else { "false" };
            tokens.push(SyntaxToken::Bool(b));
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                tokens.push(SyntaxToken::Symbol("{}"));
            } else {
                for (idx, (field, value)) in obj.iter().enumerate() {
                    if idx > 0 || !from_arr {
                        tokens.push(SyntaxToken::Indent(indent));
                    }
                    let field_token = if is_field_complex(field) {
                        SyntaxToken::Name(format!("{field:?}"))
                    } else {
                        SyntaxToken::Name(field.to_string())
                    };
                    tokens.push(field_token);
                    tokens.push(SyntaxToken::Symbol(": "));

                    if matches!(value, Value::Object(_)) {
                        tokens.push(SyntaxToken::Break);
                    }

                    let value_tokens = highlight(value, indent + 1, false);
                    tokens.extend(value_tokens);
                }
            }
            return tokens;
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                tokens.push(SyntaxToken::Symbol("[]"));
            } else {
                tokens.push(SyntaxToken::Break);
                for value in arr {
                    tokens.push(SyntaxToken::Indent(indent));
                    tokens.push(SyntaxToken::Symbol("- "));

                    let value_tokens = highlight(value, indent + 1, true);
                    tokens.extend(value_tokens);
                }
            }
            return tokens;
        }
    }

    tokens.push(SyntaxToken::Break);
    tokens
}

fn is_field_complex(s: &str) -> bool {
    string_has_escape(s)
        || s.contains(' ')
        || s.contains(':')
        || s.contains('{')
        || s.contains('}')
        || s.contains('[')
        || s.contains(']')
}

fn append_string_tokens(tokens: &mut Vec<SyntaxToken>, s: &str, indent: usize) {
    if !is_complex_string(s) {
        if !is_special_string(s) {
            tokens.push(SyntaxToken::String(s.to_string()));
            return;
        }
        // The bool and number as string should be wrapped with quotes
        let s = format!("{s:?}");
        tokens.push(SyntaxToken::String(s));
        return;
    }

    // The complex string, we use yaml's multi-line string format
    // See: <https://yaml-multiline.info>
    let lines: Vec<_> = s.lines().collect();
    tokens.push(SyntaxToken::Symbol("|"));
    tokens.push(SyntaxToken::Break);
    for line in lines {
        tokens.push(SyntaxToken::Indent(indent + 1));
        tokens.push(SyntaxToken::String(line.to_string()));
    }
}

fn is_complex_string(s: &str) -> bool {
    if !string_has_escape(s) {
        return false;
    }

    // The string like "\n\n" should not be treated as complex string.
    let trim_str = s.trim();
    if trim_str.is_empty() {
        return false;
    }

    true
}

fn is_special_string(s: &str) -> bool {
    if string_has_escape(s) {
        return true;
    }

    if s == "true" || s == "false" {
        return true;
    }

    // The number as string, like "123.4", should wrap with quotes, or yaml parser will
    // treat it as a "real" number.
    s.parse::<f64>().is_ok()
}

fn string_has_escape(s: &str) -> bool {
    s.contains('\n') || s.contains('\r') || s.contains('"') || s.contains('\'')
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_syntax_highlight() {
        let test_cases = [
            (
                include_str!("test_cases/yaml/common.yaml"),
                include_str!("test_cases/yaml/common.yaml"),
            ),
            (
                include_str!("test_cases/yaml/complex.yaml"),
                include_str!("test_cases/yaml/complex.yaml"),
            ),
        ];

        let parser = YamlParser {};
        for (raw, expect) in test_cases {
            let value = parser.parse(raw).unwrap();
            let tokens = parser.syntax_highlight(&value);
            let result = SyntaxToken::pure_text(&tokens);
            println!("{result}");
            assert_eq!(result, expect);
        }
    }
}
