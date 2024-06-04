use anyhow::{Context, Result};
use serde_json::Value;

use super::Parser;

pub(super) struct JsonParser {}

impl Parser for JsonParser {
    fn parse(&self, data: &str) -> Result<Value> {
        serde_json::from_str(data).context("parse JSON")
    }

    fn syntax_highlight(&self, value: &Value) -> String {
        // TODO: Implement syntax highlighting
        serde_json::to_string_pretty(value).expect("serialize JSON")
    }
}
