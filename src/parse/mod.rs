mod json;
mod syntax;
mod toml;
mod yaml;

pub use syntax::SyntaxToken;

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
    fn extension(&self) -> &'static str;

    fn parse(&self, data: &str) -> Result<Value>;

    fn to_string(&self, value: &Value) -> String;

    fn syntax_highlight(&self, value: &Value) -> Vec<SyntaxToken>;
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
