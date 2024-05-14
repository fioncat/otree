pub mod colors;
pub mod keys;
pub mod types;

use anyhow::Result;
use serde::Deserialize;

use self::colors::Colors;
use self::keys::Keys;
use self::types::Types;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "Colors::default")]
    pub colors: Colors,

    #[serde(default = "Types::default")]
    pub types: Types,

    #[serde(default = "Keys::default")]
    pub keys: Keys,
}

impl Config {
    pub fn default() -> Self {
        Self {
            colors: Colors::default(),
            types: Types::default(),
            keys: Keys::default(),
        }
    }

    pub fn parse(&mut self) -> Result<()> {
        self.colors.parse()?;
        self.keys.parse()?;
        Ok(())
    }
}
