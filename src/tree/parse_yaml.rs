use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;

#[inline(always)]
pub fn parse(data: &str) -> Result<Value> {
    let mut values = Vec::with_capacity(1);
    for document in serde_yml::Deserializer::from_str(data) {
        let value = Value::deserialize(document).context("parse yaml")?;
        values.push(value);
    }

    if values.is_empty() {
        bail!("no document found in yaml data");
    }

    if values.len() == 1 {
        return Ok(values.into_iter().next().unwrap());
    }

    Ok(Value::Array(values))
}

#[inline(always)]
pub fn to_string(values: &Value) -> Result<String> {
    serde_yml::to_string(values).context("serialize yaml")
}
