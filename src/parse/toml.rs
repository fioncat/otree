use anyhow::{Context, Result};
use serde_json::Value;

use super::Parser;

pub(super) struct TomlParser {}

impl Parser for TomlParser {
    fn parse(&self, data: &str) -> Result<Value> {
        toml::from_str(data).context("parse TOML")
    }

    fn syntax_highlight(&self, value: &Value) -> String {
        if let Value::Array(arr) = value {
            // TODO: Handle array toml
            return serde_json::to_string_pretty(arr).expect("serialize JSON");
        }
        toml::to_string_pretty(value).expect("serialize TOML")
    }
}
