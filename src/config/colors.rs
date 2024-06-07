use std::collections::HashMap;

use anyhow::{Context, Result};
use ratatui::style::{Style, Stylize};
use serde::{Deserialize, Serialize};

use super::Config;

macro_rules! generate_colors_parse {
    ($StructName:ident, $($field:ident),+) => {
        impl $StructName {
            pub fn parse(&mut self, palette: &std::collections::HashMap<String, String>) -> Result<()> {
                $(self.$field.parse(palette).with_context(|| format!("parse color for {}", stringify!($field)))?;)+
                Ok(())
            }
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colors {
    #[serde(default = "Colors::default_header")]
    pub header: Color,

    #[serde(default = "Colors::default_focus_boder")]
    pub focus_border: Color,

    #[serde(default = "TreeColors::default")]
    pub tree: TreeColors,

    #[serde(default = "DataColors::default")]
    pub data: DataColors,
}

generate_colors_parse!(Colors, header, tree, data, focus_border);

impl Colors {
    pub fn default() -> Self {
        Self {
            header: Self::default_header(),
            tree: TreeColors::default(),
            data: DataColors::default(),
            focus_border: Self::default_focus_boder(),
        }
    }

    fn default_header() -> Color {
        Color::new("", "", true, false)
    }

    fn default_focus_boder() -> Color {
        Color::new("magenta", "", true, false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataColors {
    #[serde(default = "Color::default")]
    pub text: Color,

    #[serde(default = "DataColors::default_border")]
    pub border: Color,

    #[serde(default = "Color::default")]
    pub symbol: Color,

    #[serde(default = "DataColors::default_name")]
    pub name: Color,

    #[serde(default = "DataColors::default_str")]
    pub str: Color,

    #[serde(default = "DataColors::default_num")]
    pub num: Color,

    #[serde(default = "DataColors::default_null")]
    pub null: Color,

    #[serde(default = "DataColors::default_bool")]
    pub bool: Color,

    #[serde(default = "DataColors::default_section")]
    pub section: Color,
}

generate_colors_parse!(DataColors, text, border, symbol, name, str, num, null, bool, section);

impl DataColors {
    fn default() -> Self {
        Self {
            text: Color::default(),
            border: Self::default_border(),
            symbol: Color::default(),
            name: Self::default_name(),
            str: Self::default_str(),
            num: Self::default_num(),
            null: Self::default_null(),
            bool: Self::default_bool(),
            section: Self::default_section(),
        }
    }

    fn default_border() -> Color {
        Color::new("blue", "", false, false)
    }

    fn default_name() -> Color {
        Color::new("yellow", "", false, true)
    }

    fn default_str() -> Color {
        Color::new("green", "", false, false)
    }

    fn default_num() -> Color {
        Color::new("red", "", false, false)
    }

    fn default_null() -> Color {
        Color::new("blue", "", false, true)
    }

    fn default_bool() -> Color {
        Color::new("red", "", true, true)
    }

    fn default_section() -> Color {
        Color::new("cyan", "", true, false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeColors {
    #[serde(default = "TreeColors::default_border")]
    pub border: Color,

    #[serde(default = "TreeColors::default_selected")]
    pub selected: Color,

    #[serde(default = "Color::default")]
    pub name: Color,

    #[serde(default = "TreeColors::default_type")]
    pub type_str: Color,

    #[serde(default = "TreeColors::default_type")]
    pub type_null: Color,

    #[serde(default = "TreeColors::default_type")]
    pub type_bool: Color,

    #[serde(default = "TreeColors::default_type")]
    pub type_num: Color,

    #[serde(default = "TreeColors::default_type")]
    pub type_arr: Color,

    #[serde(default = "TreeColors::default_type")]
    pub type_obj: Color,

    #[serde(default = "TreeColors::default_description")]
    pub description: Color,

    #[serde(default = "TreeColors::default_null")]
    pub null: Color,
}

generate_colors_parse!(
    TreeColors,
    border,
    selected,
    name,
    type_str,
    type_null,
    type_bool,
    type_num,
    type_arr,
    type_obj,
    description,
    null
);

impl TreeColors {
    fn default() -> Self {
        Self {
            border: Self::default_border(),
            selected: Self::default_selected(),
            name: Color::default(),
            type_str: Self::default_type(),
            type_null: Self::default_type(),
            type_bool: Self::default_type(),
            type_num: Self::default_type(),
            type_arr: Self::default_type(),
            type_obj: Self::default_type(),
            description: Self::default_description(),
            null: Self::default_null(),
        }
    }

    fn default_border() -> Color {
        Color::new("blue", "", false, false)
    }

    fn default_selected() -> Color {
        Color::new("black", "light_green", false, false)
    }

    fn default_type() -> Color {
        Color::new("cyan", "", true, true)
    }

    fn default_description() -> Color {
        Color::new("dark_gray", "", false, false)
    }

    fn default_null() -> Color {
        Color::new("dark_gray", "", false, true)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Color {
    pub fg: Option<String>,
    pub bg: Option<String>,

    #[serde(default = "Config::disable")]
    pub bold: bool,
    #[serde(default = "Config::disable")]
    pub italic: bool,

    #[serde(skip)]
    pub style: Style,
}

impl Color {
    fn new(fg: &str, bg: &str, bold: bool, italic: bool) -> Self {
        let fg = if fg.is_empty() {
            None
        } else {
            Some(String::from(fg))
        };
        let bg = if bg.is_empty() {
            None
        } else {
            Some(String::from(bg))
        };
        Self {
            fg,
            bg,
            bold,
            italic,
            style: Style::default(),
        }
    }

    fn parse(&mut self, palette: &HashMap<String, String>) -> Result<()> {
        let mut style = Style::default();
        if let Some(mut fg) = self.fg.as_ref() {
            if let Some(color) = palette.get(fg) {
                fg = color;
            }
            style = style.fg(fg.parse().context("parse fg color")?);
        }
        if let Some(mut bg) = self.bg.as_ref() {
            if let Some(color) = palette.get(bg) {
                bg = color;
            }
            style = style.bg(bg.parse().context("parse bg color")?);
        }
        if self.bold {
            style = style.bold();
        }
        if self.italic {
            style = style.italic();
        }
        self.style = style;
        Ok(())
    }
}
