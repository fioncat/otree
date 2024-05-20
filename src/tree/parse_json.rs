use anyhow::{Context, Result};
use serde_json::Value;

#[inline(always)]
pub fn parse(data: &str) -> Result<Value> {
    serde_json::from_str(data).context("parse json")
}

#[inline(always)]
pub fn to_string(value: &Value) -> Result<String> {
    serde_json::to_string_pretty(value).context("serialize json")
}
