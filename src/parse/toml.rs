use anyhow::{Context, Result};
use serde_json::{Map, Number, Value};
use toml::Value as TomlValue;

use super::json::highlight as json_highlight;
use super::syntax::{self, StringValue};
use super::{Parser, SyntaxToken};

pub struct TomlParser {}

impl Parser for TomlParser {
    fn extension(&self) -> &'static str {
        "toml"
    }

    fn parse(&self, data: &str) -> Result<Value> {
        let toml_value: TomlValue = toml::from_str(data).context("parse TOML")?;
        Ok(toml_value_to_json(toml_value))
    }

    fn syntax_highlight(&self, name: &str, value: &Value) -> Vec<SyntaxToken> {
        let mut section = None;
        let mut arr_complex = false;
        if let Value::Array(arr) = value {
            section = Some(name);

            for value in arr {
                match value {
                    Value::Object(_) | Value::Array(_) => {
                        arr_complex = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
        let mut tokens = highlight(value, section, false, arr_complex);
        if !tokens.is_empty() {
            // Trim the first break line
            let first_token = tokens.remove(0);
            if !matches!(first_token, SyntaxToken::Break) {
                tokens.insert(0, first_token);
            }
        }

        tokens
    }
}

fn toml_value_to_json(toml_value: TomlValue) -> Value {
    match toml_value {
        TomlValue::String(s) => Value::String(s),
        TomlValue::Integer(i) => Value::Number(Number::from(i)),
        TomlValue::Float(f) => Value::Number(Number::from_f64(f).unwrap_or(Number::from(0))),
        TomlValue::Boolean(b) => Value::Bool(b),
        TomlValue::Datetime(datetime) => Value::String(datetime.to_string()),
        TomlValue::Array(arr) => {
            let mut json_arr = Vec::with_capacity(arr.len());
            for toml_value in arr {
                let value = toml_value_to_json(toml_value);
                json_arr.push(value);
            }
            Value::Array(json_arr)
        }
        TomlValue::Table(table) => {
            let mut json_obj = Map::with_capacity(table.len());
            for (field, toml_value) in table {
                let value = toml_value_to_json(toml_value);
                json_obj.insert(field, value);
            }
            Value::Object(json_obj)
        }
    }
}

fn highlight(
    value: &Value,
    section: Option<&str>,
    from_arr: bool,
    arr_complex: bool,
) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();
    let section = section.as_ref();

    if from_arr {
        if let Some(section) = section {
            tokens.push(SyntaxToken::Break);
            tokens.push(SyntaxToken::Section(format!("[[{section}]]")));
            tokens.push(SyntaxToken::Break);
        }
    }

    match value {
        // The TOML does not support null type, let's use empty string instead
        Value::Null => tokens.push(SyntaxToken::String(String::from("\"\""))),
        Value::String(s) => {
            // TODO: Handle datetimes, they should not be quoted.
            // See: <https://toml.io/en/v1.0.0#offset-date-time>
            let value = StringValue::new(s, true);
            match value {
                StringValue::String(s) => tokens.push(SyntaxToken::String(s)),
                StringValue::MultiLines(lines) if !s.contains("'''") => {
                    tokens.push(SyntaxToken::String(String::from("'''")));
                    tokens.push(SyntaxToken::Break);
                    for line in lines {
                        tokens.push(SyntaxToken::String(line));
                        tokens.push(SyntaxToken::Break);
                    }
                    tokens.push(SyntaxToken::String(String::from("'''")));
                }
                StringValue::MultiLines(_) => tokens.push(SyntaxToken::String(format!("{s:?}"))),
            }
        }
        Value::Number(num) => tokens.push(SyntaxToken::Number(num.to_string())),
        Value::Bool(b) => {
            let b = if *b { "true" } else { "false" };
            tokens.push(SyntaxToken::Bool(b));
        }

        Value::Object(obj) => {
            if !from_arr {
                if let Some(section) = section {
                    tokens.push(SyntaxToken::Break);
                    tokens.push(SyntaxToken::Section(format!("[{section}]")));
                    tokens.push(SyntaxToken::Break);
                }
            }

            let mut complex_fields = Vec::new();
            for (field, value) in obj {
                let is_complex = is_value_complex(value);
                if is_complex {
                    complex_fields.push((field, value));
                    continue;
                }

                let field = syntax::quote_field_name(field);
                tokens.push(SyntaxToken::Name(field));
                tokens.push(SyntaxToken::Symbol(" = "));

                let value_tokens = highlight(value, None, false, false);
                tokens.extend(value_tokens);
            }

            for (field, value) in complex_fields {
                let field = syntax::quote_field_name(field);
                let child_section = match section.as_ref() {
                    Some(section) => format!("{section}.{field}"),
                    None => field,
                };

                let value_tokens = highlight(value, Some(&child_section), false, true);
                tokens.extend(value_tokens);
            }

            return tokens;
        }

        Value::Array(arr) => {
            if !arr_complex {
                // Simple array, we can use json schema.
                let json_tokens = json_highlight(value, 0, false);
                tokens.extend(json_tokens);
                return tokens;
            }

            // TOML does not support direct array. Section MUST be provided.
            debug_assert!(section.is_some());

            for value in arr {
                let value_tokens = highlight(value, section.copied(), true, false);
                tokens.extend(value_tokens);
            }

            return tokens;
        }
    }

    tokens.push(SyntaxToken::Break);
    tokens
}

fn is_value_complex(value: &Value) -> bool {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_syntax_highlight() {
        let test_cases = [
            (
                include_str!("test_cases/toml/basic.toml"),
                include_str!("test_cases/toml/basic_highlight.toml"),
            ),
            (
                include_str!("test_cases/toml/array_of_objects.toml"),
                include_str!("test_cases/toml/array_of_objects.toml"),
            ),
            (
                include_str!("test_cases/toml/multilines.toml"),
                include_str!("test_cases/toml/multilines_highlight.toml"),
            ),
            (
                include_str!("test_cases/toml/2d_array.toml"),
                include_str!("test_cases/toml/2d_array_highlight.toml"),
            ),
        ];

        let parser = TomlParser {};
        for (raw, expect) in test_cases {
            let value = parser.parse(raw).unwrap();
            let tokens = parser.syntax_highlight("", &value);
            let result = SyntaxToken::pure_text(&tokens);
            assert_eq!(result, expect);

            let highlight_value = parser.parse(&result).unwrap();
            assert_eq!(value, highlight_value);
        }
    }
}
