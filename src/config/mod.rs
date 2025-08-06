pub mod colors;
pub mod keys;
pub mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs, io};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use self::colors::Colors;
use self::keys::Keys;
use self::types::Types;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "Editor::default")]
    pub editor: Editor,

    #[serde(default = "Data::default")]
    pub data: Data,

    #[serde(default = "Layout::default")]
    pub layout: Layout,

    #[serde(default = "Header::default")]
    pub header: Header,

    #[serde(default = "Footer::default")]
    pub footer: Footer,

    #[serde(default = "Filter::default")]
    pub filter: Filter,

    #[serde(default = "Config::empty_map")]
    pub palette: HashMap<String, String>,

    #[serde(default = "Colors::default")]
    pub colors: Colors,

    #[serde(default = "Types::default")]
    pub types: Types,

    #[serde(default = "Keys::default")]
    pub keys: Keys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Editor {
    #[serde(default = "Editor::default_program")]
    pub program: String,

    #[serde(default = "Editor::default_args")]
    pub args: Vec<String>,

    #[serde(default = "Editor::default_dir")]
    pub dir: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    #[serde(default = "Config::disable")]
    pub disable: bool,

    #[serde(default = "Header::default_format")]
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footer {
    #[serde(default = "Config::disable")]
    pub disable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    #[serde(default = "Config::disable")]
    pub disable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    #[serde(default = "Config::disable")]
    pub disable_highlight: bool,
    #[serde(default = "Config::default_max_data_size")]
    pub max_data_size: usize,
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

        self.validate_palette()?;
        self.colors.parse(&self.palette)?;
        self.keys.parse()?;
        Ok(())
    }

    fn validate_palette(&self) -> Result<()> {
        use ratatui::style::Color;
        for (key, color) in self.palette.iter() {
            if let Err(err) = color.parse::<Color>() {
                return Err(err).with_context(|| format!("validate palette color '{key}'"));
            }
        }
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
            editor: Editor::default(),
            data: Data::default(),
            layout: Layout::default(),
            header: Header::default(),
            footer: Footer::default(),
            filter: Filter::default(),
            palette: Self::empty_map(),
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

    const fn disable() -> bool {
        false
    }

    const fn default_max_data_size() -> usize {
        30
    }

    fn empty_map() -> HashMap<String, String> {
        HashMap::new()
    }
}

impl Editor {
    fn default() -> Self {
        Self {
            program: Self::default_program(),
            args: Self::default_args(),
            dir: Self::default_dir(),
        }
    }

    fn default_program() -> String {
        if let Some(editor) = env::var_os("EDITOR") {
            return editor.to_string_lossy().to_string();
        }
        String::from("vim")
    }

    fn default_args() -> Vec<String> {
        vec![String::from("{file}")]
    }

    fn default_dir() -> String {
        String::from("/tmp")
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

impl Header {
    fn default() -> Self {
        Self {
            disable: Config::disable(),
            format: Self::default_format(),
        }
    }

    fn default_format() -> String {
        "{version} - {data_source} ({content_type}) - {data_size}".to_string()
    }
}

impl Footer {
    fn default() -> Self {
        Self {
            disable: Config::disable(),
        }
    }
}

impl Filter {
    fn default() -> Self {
        Self {
            disable: Config::disable(),
        }
    }
}

impl Data {
    fn default() -> Self {
        Self {
            disable_highlight: Config::disable(),
            max_data_size: Config::default_max_data_size(),
        }
    }
}
