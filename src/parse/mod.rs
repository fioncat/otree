mod any;
mod hcl;
mod json;
mod jsonl;
mod syntax;
mod toml;
mod xml;
mod yaml;

pub use syntax::SyntaxToken;

use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use serde_json::Value;
use strum::EnumIter;

#[derive(Debug, Clone, Copy, ValueEnum, EnumIter)]
pub enum ContentType {
    Json,
    Yaml,
    Toml,
    Xml,

    /// See: <https://github.com/hashicorp/hcl>
    Hcl,

    /// Useful for some logs file: <https://jsonlines.org/>
    // TODO: check out json-seq as specified in RFC7464 (https://datatracker.ietf.org/doc/html/rfc7464)?
    // basically jsonl but every json object is prefixed with 0x1e
    Jsonl,

    #[clap(skip)]
    Any,
}

pub trait Parser {
    fn extension(&self) -> &'static str;

    fn allow_array_root(&self) -> bool;

    fn parse(&self, data: &str) -> Result<Value>;

    fn syntax_highlight(&self, name: &str, value: &Value) -> Vec<SyntaxToken>;

    fn parse_root(&self, validator: Option<&dyn Parser>, data: &[u8]) -> Result<Value> {
        let data = String::from_utf8(data.to_vec()).context("parse content utf8")?;
        let value = self.parse(&data)?;

        match value {
            Value::Object(_) => {}
            Value::Array(_) => {
                let allow =
                    validator.map_or_else(|| self.allow_array_root(), Parser::allow_array_root);
                if !allow {
                    let schema = validator.map_or_else(|| self.extension(), Parser::extension);
                    bail!("schema '{schema}' does not allow array as root");
                }
            }
            _ => bail!("root value must be either object or array"),
        }

        Ok(value)
    }
}

impl ContentType {
    pub fn new_parser(self) -> Box<dyn Parser> {
        match self {
            Self::Json => Box::new(json::JsonParser {}),
            Self::Yaml => Box::new(yaml::YamlParser {}),
            Self::Toml => Box::new(toml::TomlParser {}),
            Self::Xml => Box::new(xml::XmlParser {}),
            Self::Hcl => Box::new(hcl::HclParser {}),
            Self::Jsonl => Box::new(jsonl::JsonlParser {}),
            Self::Any => Box::new(any::AnyParser::new()),
        }
    }
}
