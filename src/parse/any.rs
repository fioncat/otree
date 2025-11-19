use std::cell::RefCell;

use anyhow::{bail, Result};
use serde_json::Value;
use strum::IntoEnumIterator;

use super::{ContentType, Parser, SyntaxToken};

pub struct AnyParser {
    inner: RefCell<Option<Box<dyn Parser>>>,
}

impl AnyParser {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(None),
        }
    }

    pub fn set_parser(&self, parser: Box<dyn Parser>) {
        *self.inner.borrow_mut() = Some(parser);
    }
}

impl Parser for AnyParser {
    fn extension(&self) -> &'static str {
        self.inner.borrow().as_ref().unwrap().extension()
    }

    fn allow_array_root(&self) -> bool {
        self.inner.borrow().as_ref().unwrap().allow_array_root()
    }

    fn parse(&self, data: &str) -> Result<Value> {
        self.inner.borrow().as_ref().unwrap().parse(data)
    }

    fn syntax_highlight(&self, name: &str, value: &Value) -> Vec<SyntaxToken> {
        self.inner
            .borrow()
            .as_ref()
            .unwrap()
            .syntax_highlight(name, value)
    }

    fn parse_root(&self, validator: Option<&dyn Parser>, data: &[u8]) -> Result<Value> {
        for ct in ContentType::iter() {
            if matches!(ct, ContentType::Any) {
                continue;
            }

            let parser = ct.new_parser();
            if let Ok(value) = parser.parse_root(validator, data) {
                self.set_parser(parser);
                return Ok(value);
            }
        }

        bail!("unable to parse content with any supported format")
    }
}
