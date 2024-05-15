pub mod colors;
pub mod keys;
pub mod types;

use std::path::PathBuf;
use std::{env, fs, io};

use anyhow::{Context, Result};
use serde::Deserialize;

use self::colors::Colors;
use self::keys::Keys;
use self::types::Types;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_layout")]
    pub layout: LayoutDirection,

    #[serde(default = "Colors::default")]
    pub colors: Colors,

    #[serde(default = "Types::default")]
    pub types: Types,

    #[serde(default = "Keys::default")]
    pub keys: Keys,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum LayoutDirection {
    #[serde(rename = "vertical")]
    Vertical,
    #[serde(rename = "horizontal")]
    Horizontal,
}

impl Config {
    pub fn load(path: Option<String>) -> Result<Self> {
        let mut cfg = Self::read(path)?;
        cfg.parse().context("parse config")?;
        Ok(cfg)
    }

    fn read(path: Option<String>) -> Result<Self> {
        let path = Self::get_path(path).context("get config path")?;
        if path.is_none() {
            return Ok(Self::default());
        }
        let path = path.unwrap();

        let data = fs::read_to_string(&path)
            .with_context(|| format!("read config file '{}'", path.display()))?;

        toml::from_str(&data).context("parse config toml")
    }

    fn parse(&mut self) -> Result<()> {
        self.colors.parse()?;
        self.keys.parse()?;
        Ok(())
    }

    fn get_path(path: Option<String>) -> Result<Option<PathBuf>> {
        if let Some(path) = path {
            return Ok(Some(PathBuf::from(path)));
        }

        let path = env::var_os("OTREE_CONFIG");
        if let Some(path) = path {
            return Ok(Some(PathBuf::from(path)));
        }

        let home = dirs::home_dir();
        if home.is_none() {
            return Ok(None);
        }
        let home = home.unwrap();

        let path = home.join(".config").join("otree.toml");
        match fs::metadata(&path) {
            Ok(_) => Ok(Some(path)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err)
                .with_context(|| format!("get metadata for config file '{}'", path.display())),
        }
    }

    fn default() -> Self {
        Self {
            layout: Self::default_layout(),
            colors: Colors::default(),
            types: Types::default(),
            keys: Keys::default(),
        }
    }

    fn default_layout() -> LayoutDirection {
        LayoutDirection::Vertical
    }
}
