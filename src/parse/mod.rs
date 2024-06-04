mod json;
mod toml;
mod yaml;

use anyhow::Result;
use clap::ValueEnum;
use serde_json::Value;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ContentType {
    Json,
    Yaml,
    Toml,
}

pub trait Parser {
    fn parse(&self, data: &str) -> Result<Value>;

    // TODO: Returns `Text` to implement highlighting.
    fn syntax_highlight(&self, value: &Value) -> String;
}

impl ContentType {
    pub fn new_parser(&self) -> Box<dyn Parser> {
        match self {
            Self::Json => Box::new(json::JsonParser {}),
            Self::Yaml => Box::new(yaml::YamlParser {}),
            Self::Toml => Box::new(toml::TomlParser {}),
        }
    }
}
