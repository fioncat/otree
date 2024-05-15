mod parse_json;
mod parse_toml;
mod parse_yaml;

use std::borrow::Cow;
use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::ValueEnum;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use serde_json::Value;
use tui_tree_widget::TreeItem;

use crate::config::Config;

pub struct Tree<'a> {
    pub items: Vec<TreeItem<'a, String>>,
    pub details: HashMap<String, Detail>,
}

#[derive(Debug, Clone)]
pub struct Detail {
    pub value: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ContentType {
    Json,
    Toml,
    Yaml,
}

struct TreeItemValue<'a> {
    type_text: &'a str,
    type_style: Style,

    description: Cow<'static, str>,

    detail: Detail,

    children: Option<Vec<TreeItem<'a, String>>>,
}

impl<'a> Tree<'a> {
    pub fn parse(cfg: &'a Config, data: &str, content_type: ContentType) -> Result<Self> {
        let value = content_type.parse(data)?;
        let mut details: HashMap<String, Detail> = HashMap::new();

        let items: Vec<TreeItem<String>> = if let Value::Array(arr) = value {
            let mut items = Vec::with_capacity(arr.len());
            for (idx, value) in arr.into_iter().enumerate() {
                let item = Self::parse_value(
                    cfg,
                    vec![],
                    idx.to_string(),
                    value,
                    &mut details,
                    content_type,
                )?;
                items.push(item);
            }
            items
        } else if let Value::Object(obj) = value {
            let mut items = Vec::with_capacity(obj.len());
            for (field, value) in obj {
                let item =
                    Self::parse_value(cfg, vec![], field, value, &mut details, content_type)?;
                items.push(item);
            }
            items
        } else {
            vec![Self::parse_value(
                cfg,
                vec![],
                String::from("root"),
                value,
                &mut details,
                content_type,
            )?]
        };

        Ok(Self { items, details })
    }

    fn parse_value(
        cfg: &'a Config,
        parent: Vec<String>,
        name: String,
        value: Value,
        details: &mut HashMap<String, Detail>,
        content_type: ContentType,
    ) -> Result<TreeItem<'a, String>> {
        let value = TreeItemValue::parse(cfg, &parent, &name, value, details, content_type)?;

        let TreeItemValue {
            type_text,
            type_style,
            description,
            detail,
            children,
        } = value;

        let path = if parent.is_empty() {
            name.clone()
        } else {
            format!("{}/{name}", parent.join("/"))
        };
        details.insert(path, detail);

        let line = Line::from(vec![
            Span::styled(name.clone(), cfg.colors.item.name.style),
            Span::raw(" "),
            Span::styled(type_text.to_string(), type_style),
            Span::raw(" "),
            Span::styled(description.to_string(), cfg.colors.item.description.style),
        ]);

        let item = match children {
            Some(children) => TreeItem::new(name, line, children).unwrap(),
            None => TreeItem::new_leaf(name, line),
        };
        Ok(item)
    }
}

impl ContentType {
    fn parse(&self, data: &str) -> Result<Value> {
        match self {
            Self::Json => parse_json::parse(data),
            Self::Toml => parse_toml::parse(data),
            Self::Yaml => parse_yaml::parse(data),
        }
    }

    fn serialize(&self, value: &Value) -> Result<String> {
        match self {
            Self::Json => parse_json::to_string(value),
            Self::Toml => parse_toml::to_string(value),
            Self::Yaml => parse_yaml::to_string(value),
        }
    }
}

impl<'a> TreeItemValue<'a> {
    fn parse(
        cfg: &'a Config,
        parent: &[String],
        name: &String,
        value: Value,
        details: &mut HashMap<String, Detail>,
        content_type: ContentType,
    ) -> Result<Self> {
        match value {
            Value::Null => Ok(Self {
                type_text: cfg.types.null.as_str(),
                type_style: cfg.colors.item.type_null.style,
                description: Cow::Borrowed("null"),
                detail: Detail {
                    value: String::new(),
                },
                children: None,
            }),
            Value::String(s) => Ok(Self {
                type_text: cfg.types.str.as_str(),
                type_style: cfg.colors.item.type_str.style,
                description: Cow::Owned(format!("= {s:?}")),
                detail: Detail { value: s },
                children: None,
            }),
            Value::Number(num) => Ok(Self {
                type_text: cfg.types.num.as_str(),
                type_style: cfg.colors.item.type_num.style,
                description: Cow::Owned(format!("= {num}")),
                detail: Detail {
                    value: num.to_string(),
                },
                children: None,
            }),
            Value::Bool(b) => Ok(Self {
                type_text: cfg.types.bool.as_str(),
                type_style: cfg.colors.item.type_bool.style,
                description: Cow::Owned(format!("= {b}")),
                detail: Detail {
                    value: b.to_string(),
                },
                children: None,
            }),
            Value::Array(arr) => {
                let detail = content_type
                    .serialize(&Value::Array(arr.clone()))
                    .with_context(|| {
                        format!("serialize for array item '{}/{name}'", parent.join("/"))
                    })?;

                let word = if arr.len() > 1 { "items" } else { "item" };
                let description = Cow::Owned(format!("[ {} {word} ]", arr.len()));
                let mut children = Vec::with_capacity(arr.len());
                for (idx, item) in arr.into_iter().enumerate() {
                    let mut child_parent = parent.to_vec();
                    child_parent.push(name.clone());

                    let child = Tree::parse_value(
                        cfg,
                        child_parent,
                        idx.to_string(),
                        item,
                        details,
                        content_type,
                    )?;
                    children.push(child);
                }
                Ok(Self {
                    type_text: cfg.types.arr.as_str(),
                    type_style: cfg.colors.item.type_arr.style,
                    description,
                    detail: Detail { value: detail },
                    children: Some(children),
                })
            }
            Value::Object(obj) => {
                let detail = content_type
                    .serialize(&Value::Object(obj.clone()))
                    .with_context(|| {
                        format!("serialize for object item '{}/{name}'", parent.join("/"))
                    })?;

                let word = if obj.len() > 1 { "fields" } else { "field" };
                let description = Cow::Owned(format!("{{ {} {word} }}", obj.len()));

                let mut children = Vec::with_capacity(obj.len());
                for (field, item) in obj {
                    let mut child_parent = parent.to_vec();
                    child_parent.push(name.clone());

                    let child =
                        Tree::parse_value(cfg, child_parent, field, item, details, content_type)?;
                    children.push(child);
                }
                Ok(Self {
                    type_text: cfg.types.obj.as_str(),
                    type_style: cfg.colors.item.type_obj.style,
                    description,
                    detail: Detail { value: detail },
                    children: Some(children),
                })
            }
        }
    }
}
