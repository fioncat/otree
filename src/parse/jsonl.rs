use anyhow::{Context, Result};
use serde_json::Value;

use super::json;
use super::{Parser, SyntaxToken};

pub struct JsonlParser;

impl Parser for JsonlParser {
    fn extension(&self) -> &'static str {
        "json"
    }

    fn parse(&self, data: &str) -> Result<Value> {
        // Each line of the JSONL file is a JSON object
        let lines: Vec<&str> = data.lines().collect();
        let mut objects = Vec::with_capacity(lines.len());
        for (idx, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                // let's skip those empty lines
                continue;
            }
            let object = serde_json::from_str(line)
                .with_context(|| format!("parse JSON object at line {}", idx + 1))?;
            objects.push(object);
        }
        Ok(Value::Array(objects))
    }

    fn to_string(&self, value: &Value) -> String {
        serde_json::to_string_pretty(value).expect("serialize JSON")
    }

    fn syntax_highlight(&self, value: &Value) -> Vec<SyntaxToken> {
        json::highlight(value, 0, false)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_syntax_highlight() {
        let test_cases = [
            (
                include_str!("test_cases/jsonl/csv.jsonl"),
                include_str!("test_cases/jsonl/csv_highlight.json"),
            ),
            (
                include_str!("test_cases/jsonl/objects.jsonl"),
                include_str!("test_cases/jsonl/objects_highlight.json"),
            ),
            (
                include_str!("test_cases/jsonl/complex_objects.jsonl"),
                include_str!("test_cases/jsonl/complex_objects_highlight.json"),
            ),
        ];

        let parser = JsonlParser {};
        for (raw, expect) in test_cases {
            let value = parser.parse(raw).unwrap();
            let tokens = parser.syntax_highlight(&value);
            let result = SyntaxToken::pure_text(&tokens);
            assert_eq!(result, expect);
        }
    }
}
