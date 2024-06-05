use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::syntax::{self, StringValue};
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
        if let Value::Array(arr) = value {
            if arr.is_empty() {
                return vec![SyntaxToken::Symbol("[]")];
            }

            // YAML Multi Documents
            // See: <https://gettaurus.org/docs/YAMLTutorial/#YAML-Multi-Documents>
            let mut tokens = Vec::new();
            for value in arr {
                tokens.push(SyntaxToken::Symbol("---"));
                tokens.push(SyntaxToken::Break);
                let document_tokens = highlight(value, 0, false);
                tokens.extend(document_tokens);
            }
            return tokens;
        }

        highlight(value, 0, false)
    }
}

fn highlight(value: &Value, indent: usize, from_arr: bool) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    match value {
        Value::Null => tokens.push(SyntaxToken::Null("null")),
        Value::String(s) => {
            let value = StringValue::new(s, false);
            match value {
                StringValue::String(s) => tokens.push(SyntaxToken::String(s)),
                StringValue::MultiLines(lines) => {
                    tokens.push(SyntaxToken::Symbol("|"));
                    tokens.push(SyntaxToken::Break);
                    for line in lines {
                        if line.is_empty() {
                            tokens.push(SyntaxToken::Break);
                            continue;
                        }
                        tokens.push(SyntaxToken::Indent(indent));
                        tokens.push(SyntaxToken::String(line));
                        tokens.push(SyntaxToken::Break);
                    }

                    // MultiLines done, don't need to append the last break.
                    return tokens;
                }
            }
        }
        Value::Number(num) => tokens.push(SyntaxToken::Number(num.to_string())),
        Value::Bool(b) => {
            let b = if *b { "true" } else { "false" };
            tokens.push(SyntaxToken::Bool(b));
        }
        Value::Object(obj) => {
            if !obj.is_empty() {
                for (idx, (field, value)) in obj.iter().enumerate() {
                    if idx > 0 || !from_arr {
                        tokens.push(SyntaxToken::Indent(indent));
                    }
                    let field = syntax::quote_field_name(field);
                    tokens.push(SyntaxToken::Name(field));

                    let is_value_complex = match value {
                        Value::Object(obj) => !obj.is_empty(),
                        Value::Array(arr) => !arr.is_empty(),
                        _ => false,
                    };

                    if is_value_complex {
                        tokens.push(SyntaxToken::Symbol(":"));
                        tokens.push(SyntaxToken::Break);
                    } else {
                        tokens.push(SyntaxToken::Symbol(": "));
                    }

                    let value_tokens = highlight(value, indent + 1, false);
                    tokens.extend(value_tokens);
                }
                return tokens;
            }
            tokens.push(SyntaxToken::Symbol("{}"));
        }
        Value::Array(arr) => {
            if !arr.is_empty() {
                for (idx, value) in arr.iter().enumerate() {
                    if idx > 0 || !from_arr {
                        tokens.push(SyntaxToken::Indent(indent));
                    }

                    tokens.push(SyntaxToken::Symbol("- "));

                    let value_tokens = highlight(value, indent + 1, true);
                    tokens.extend(value_tokens);
                }
                return tokens;
            }
            tokens.push(SyntaxToken::Symbol("[]"));
        }
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
                include_str!("test_cases/yaml/common.yaml"),
                include_str!("test_cases/yaml/common_highlight.yaml"),
            ),
            (
                include_str!("test_cases/yaml/special_strings.yaml"),
                include_str!("test_cases/yaml/special_strings.yaml"),
            ),
            (
                include_str!("test_cases/yaml/nested_object.yaml"),
                include_str!("test_cases/yaml/nested_object_highlight.yaml"),
            ),
            (
                include_str!("test_cases/yaml/array_of_objects.yaml"),
                include_str!("test_cases/yaml/array_of_objects_highlight.yaml"),
            ),
            (
                include_str!("test_cases/yaml/arrays.yaml"),
                include_str!("test_cases/yaml/arrays_highlight.yaml"),
            ),
            (
                include_str!("test_cases/yaml/multilines.yaml"),
                include_str!("test_cases/yaml/multilines_highlight.yaml"),
            ),
            (
                include_str!("test_cases/yaml/multidocs.yaml"),
                include_str!("test_cases/yaml/multidocs_highlight.yaml"),
            ),
            (
                include_str!("test_cases/yaml/empty.yaml"),
                include_str!("test_cases/yaml/empty.yaml"),
            ),
        ];

        let parser = YamlParser {};
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
