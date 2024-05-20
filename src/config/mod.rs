pub mod colors;
pub mod keys;
pub mod types;

use std::path::PathBuf;
use std::{env, fs, io};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use self::colors::Colors;
use self::keys::Keys;
use self::types::Types;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "Layout::default")]
    pub layout: Layout,

    #[serde(default = "Colors::default")]
    pub colors: Colors,

    #[serde(default = "Types::default")]
    pub types: Types,

    #[serde(default = "Keys::default")]
    pub keys: Keys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    #[serde(default = "Layout::default_direction")]
    pub direction: LayoutDirection,

    #[serde(default = "Layout::default_tree_size")]
    pub tree_size: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LayoutDirection {
    #[serde(rename = "vertical")]
    Vertical,
    #[serde(rename = "horizontal")]
    Horizontal,
}

impl Config {
    pub const MIN_LAYOUT_TREE_SIZE: u16 = 10;
    pub const MAX_LAYOUT_TREE_SIZE: u16 = 80;

    pub fn load(path: Option<String>) -> Result<Self> {
        let path = Self::get_path(path).context("get config path")?;
        if path.is_none() {
            return Ok(Self::default());
        }
        let path = path.unwrap();

        let data = fs::read_to_string(&path)
            .with_context(|| format!("read config file '{}'", path.display()))?;

        toml::from_str(&data).context("parse config toml")
    }

    pub fn parse(&mut self) -> Result<()> {
        if self.layout.tree_size < Self::MIN_LAYOUT_TREE_SIZE
            || self.layout.tree_size > Self::MAX_LAYOUT_TREE_SIZE
        {
            bail!(
                "invalid layout tree size, should be between {} and {}",
                Self::MIN_LAYOUT_TREE_SIZE,
                Self::MAX_LAYOUT_TREE_SIZE
            );
        }

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

    pub fn default() -> Self {
        Self {
            layout: Layout::default(),
            colors: Colors::default(),
            types: Types::default(),
            keys: Keys::default(),
        }
    }

    pub fn show(&self) -> Result<()> {
        let toml = toml::to_string(self).context("serialize config to toml")?;
        println!("{toml}");
        Ok(())
    }
}

impl Layout {
    fn default() -> Self {
        Self {
            direction: Self::default_direction(),
            tree_size: Self::default_tree_size(),
        }
    }

    fn default_direction() -> LayoutDirection {
        LayoutDirection::Horizontal
    }

    fn default_tree_size() -> u16 {
        40
    }
}
