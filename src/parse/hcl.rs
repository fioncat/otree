use anyhow::{Context, Result};
use serde_json::{Map, Value};

use super::json::highlight as json_highlight;
use super::syntax;
use super::{Parser, SyntaxToken};

pub struct HclParser {}

impl Parser for HclParser {
    fn extension(&self) -> &'static str {
        "hcl"
    }

    fn allow_array_root(&self) -> bool {
        false
    }

    fn parse(&self, data: &str) -> Result<Value> {
        hcl::from_str(data).context("parse HCL")
    }

    fn syntax_highlight(&self, name: &str, value: &Value) -> Vec<SyntaxToken> {
        highlight(name, value.clone(), 0)
    }
}

fn highlight(name: &str, value: Value, indent: usize) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    if !syntax::is_value_complex(&value) {
        if !name.is_empty() {
            let name = syntax::quote_field_name(name);
            tokens.push(SyntaxToken::Indent(indent));
            tokens.push(SyntaxToken::Name(name));
            tokens.push(SyntaxToken::Symbol(" = "));
        }

        match value {
            Value::String(s) => tokens.push(SyntaxToken::String(format!("{s:?}"))),
            Value::Null => tokens.push(SyntaxToken::Null("null")),
            Value::Number(num) => tokens.push(SyntaxToken::Number(num.to_string())),
            Value::Bool(b) => {
                let b = if b { "true" } else { "false" };
                tokens.push(SyntaxToken::Bool(b));
            }
            Value::Array(_) => {
                let json_tokens = json_highlight(&value, indent, false);
                tokens.extend(json_tokens);
                return tokens;
            }
            Value::Object(_) => unreachable!(),
        }

        tokens.push(SyntaxToken::Break);

        return tokens;
    }

    match value {
        Value::Object(obj) => {
            if !name.is_empty() {
                let name = syntax::quote_field_name(name);
                tokens.push(SyntaxToken::Indent(indent));
                tokens.push(SyntaxToken::Name(name.clone()));
                tokens.push(SyntaxToken::Symbol(" {"));
                tokens.push(SyntaxToken::Break);
            }

            let child_indent = if name.is_empty() { indent } else { indent + 1 };
            let mut complex_field = Map::new();

            for (child_key, child_value) in obj {
                if !syntax::is_value_complex(&child_value) {
                    // TODO: For simple fields, HCL writing specifications usually perform
                    // alignment operations. We should simulate alignment for name.
                    let child_tokens = highlight(&child_key, child_value, child_indent);
                    tokens.extend(child_tokens);
                    continue;
                }
                complex_field.insert(child_key, child_value);
            }

            for (child_key, child_value) in complex_field {
                let child_tokens = highlight(&child_key, child_value, child_indent);
                tokens.extend(child_tokens);
            }

            if !name.is_empty() {
                tokens.push(SyntaxToken::Indent(indent));
                tokens.push(SyntaxToken::Symbol("}"));
                tokens.push(SyntaxToken::Break);
            }
        }
        Value::Array(arr) => {
            for child_item in arr {
                let child_tokens = highlight(name, child_item, indent);
                tokens.extend(child_tokens);
            }
        }
        _ => unreachable!(),
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlight() {
        let test_cases = [
            (
                include_str!("test_cases/hcl/basic.json"),
                include_str!("test_cases/hcl/basic.hcl"),
            ),
            (
                include_str!("test_cases/hcl/object.json"),
                include_str!("test_cases/hcl/object.hcl"),
            ),
            (
                include_str!("test_cases/hcl/array.json"),
                include_str!("test_cases/hcl/array.hcl"),
            ),
            (
                include_str!("test_cases/hcl/array_of_objects.json"),
                include_str!("test_cases/hcl/array_of_objects.hcl"),
            ),
            (
                include_str!("test_cases/hcl/nested.json"),
                include_str!("test_cases/hcl/nested.hcl"),
            ),
            (
                include_str!("test_cases/hcl/empty.json"),
                include_str!("test_cases/hcl/empty.hcl"),
            ),
        ];

        let parser = HclParser {};
        for (raw, expect) in test_cases {
            let value: Value = serde_json::from_str(raw).unwrap();
            let tokens = parser.syntax_highlight("", &value);
            let result = SyntaxToken::pure_text(&tokens);
            assert_eq!(result, expect);
        }
    }
}
