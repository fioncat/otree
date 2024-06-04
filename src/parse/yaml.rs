use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::Parser;

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

    fn syntax_highlight(&self, value: &Value) -> Vec<super::SyntaxToken> {
        todo!()
    }
}
