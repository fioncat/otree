use anyhow::{Context, Result};
use serde_json::Value;

#[inline(always)]
pub fn parse(data: &str) -> Result<Value> {
    toml::from_str(data).context("parse toml")
}

#[inline(always)]
pub fn to_string(value: &Value) -> Result<String> {
    if let Value::Array(arr) = value {
        // TODO: Handle array toml
        return serde_json::to_string_pretty(arr).context("serialize json");
    }
    toml::to_string_pretty(value).context("serialize toml")
}
