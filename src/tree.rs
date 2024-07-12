use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::Result;
use ratatui::text::{Line, Span, Text};
use serde_json::Value;
use tui_tree_widget::TreeItem;

use crate::config::Config;
use crate::parse::{ContentType, Parser, SyntaxToken};

pub struct Tree<'a> {
    pub parser: Rc<Box<dyn Parser>>,

    pub items: Vec<TreeItem<'static, String>>,
    pub values: HashMap<String, Rc<ItemValue>>,

    cfg: &'a Config,
}

pub struct ItemValue {
    pub name: String,
    pub value: Value,

    pub data: Data,
}

pub struct Data {
    pub display: Display,
    pub columns: usize,
    pub rows: usize,
}

pub enum Display {
    Raw(Cow<'static, str>),
    Highlight(Vec<SyntaxToken>),
}

#[derive(Debug, Clone, Copy)]
enum FieldType {
    Null,
    Num,
    Bool,
    Str,
    Obj,
    Arr,
}

impl<'a> Tree<'a> {
    pub fn parse(cfg: &'a Config, data: &str, content_type: ContentType) -> Result<Self> {
        let parser = content_type.new_parser();
        let value = parser.parse(data)?;
        Ok(Self::from_value(cfg, value, Rc::new(parser)))
    }

    pub fn from_value(cfg: &'a Config, value: Value, parser: Rc<Box<dyn Parser>>) -> Self {
        let mut tree = Self {
            parser,
            items: vec![],
            values: HashMap::new(),
            cfg,
        };

        // The root value needs to be expanded directly, since we don't want to see a
        // `root` item in the tree.
        let items: Vec<TreeItem<String>> = match value {
            Value::Array(arr) => {
                let mut items = Vec::with_capacity(arr.len());
                for (idx, value) in arr.into_iter().enumerate() {
                    let item = tree.build_item(vec![], idx.to_string(), value);
                    items.push(item);
                }
                items
            }
            Value::Object(obj) => {
                let mut items = Vec::with_capacity(obj.len());
                for (field, value) in obj {
                    let item = tree.build_item(vec![], field, value);
                    items.push(item);
                }
                items
            }
            _ => {
                vec![tree.build_item(vec![], String::from("root"), value)]
            }
        };
        tree.items = items;
        tree
    }

    pub fn get_value(&self, path: &str) -> Option<Rc<ItemValue>> {
        self.values.get(path).cloned()
    }

    pub fn get_parser(&self) -> Rc<Box<dyn Parser>> {
        Rc::clone(&self.parser)
    }

    fn build_item(
        &mut self,
        parent: Vec<String>,
        name: String,
        value: Value,
    ) -> TreeItem<'static, String> {
        let path = if parent.is_empty() {
            name.clone()
        } else {
            format!("{}/{name}", parent.join("/"))
        };

        let raw_value = value.clone();
        let raw_name = name.clone();
        let (item, value) = match value {
            Value::Null => (
                TreeItem::new_leaf(
                    raw_name.clone(),
                    self.build_item_text(name, FieldType::Null, Cow::Borrowed("null")),
                ),
                ItemValue {
                    name: raw_name,
                    value: raw_value,
                    data: Data::null(self.cfg),
                },
            ),
            Value::String(s) => {
                let description = format!("= {s:?}");
                let text = self.build_item_text(name, FieldType::Str, Cow::Owned(description));
                (
                    TreeItem::new_leaf(raw_name.clone(), text),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        data: Data::string(self.cfg, s),
                    },
                )
            }
            Value::Number(num) => {
                let description = format!("= {num}");
                let text = self.build_item_text(name, FieldType::Num, Cow::Owned(description));
                (
                    TreeItem::new_leaf(raw_name.clone(), text),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        data: Data::number(self.cfg, num.to_string()),
                    },
                )
            }
            Value::Bool(b) => {
                let description = if b { "= true" } else { "= false" };
                let text = self.build_item_text(name, FieldType::Bool, Cow::Borrowed(description));
                (
                    TreeItem::new_leaf(raw_name.clone(), text),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        data: Data::bool(self.cfg, b),
                    },
                )
            }
            Value::Array(arr) => {
                let description = format!(
                    "[ {} {} ]",
                    arr.len(),
                    if arr.len() > 1 { "items" } else { "item" }
                );
                let text = self.build_item_text(name, FieldType::Arr, Cow::Owned(description));
                let data = if self.cfg.data.disable_highlight {
                    Data::raw(Cow::Owned(self.parser.to_string(&raw_value)))
                } else {
                    Data::highlight(self.parser.syntax_highlight(&raw_value))
                };

                let mut children = Vec::with_capacity(arr.len());
                for (idx, item) in arr.into_iter().enumerate() {
                    let mut child_parent = parent.to_vec();
                    child_parent.push(raw_name.clone());

                    let child = self.build_item(child_parent, idx.to_string(), item);
                    children.push(child);
                }

                (
                    TreeItem::new(raw_name.clone(), text, children).unwrap(),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        data,
                    },
                )
            }
            Value::Object(obj) => {
                let description = format!(
                    "{{ {} {} }}",
                    obj.len(),
                    if obj.len() > 1 { "fields" } else { "field" }
                );
                let text = self.build_item_text(name, FieldType::Obj, Cow::Owned(description));
                let data = if self.cfg.data.disable_highlight {
                    Data::raw(Cow::Owned(self.parser.to_string(&raw_value)))
                } else {
                    Data::highlight(self.parser.syntax_highlight(&raw_value))
                };

                let mut children = Vec::with_capacity(obj.len());
                for (field, item) in obj {
                    let mut child_parent = parent.to_vec();
                    child_parent.push(raw_name.clone());

                    let child = self.build_item(child_parent, field, item);
                    children.push(child);
                }
                (
                    TreeItem::new(raw_name.clone(), text, children).unwrap(),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        data,
                    },
                )
            }
        };

        let value = Rc::new(value);
        self.values.insert(path, value);
        item
    }

    fn build_item_text(
        &self,
        name: String,
        field_type: FieldType,
        description: Cow<'static, str>,
    ) -> Text<'static> {
        // TODO: We can share field type to save memory.
        let (type_str, type_style) = match field_type {
            FieldType::Null => (
                self.cfg.types.null.clone(),
                self.cfg.colors.tree.type_null.style,
            ),
            FieldType::Num => (
                self.cfg.types.num.clone(),
                self.cfg.colors.tree.type_num.style,
            ),
            FieldType::Bool => (
                self.cfg.types.bool.clone(),
                self.cfg.colors.tree.type_bool.style,
            ),
            FieldType::Str => (
                self.cfg.types.str.clone(),
                self.cfg.colors.tree.type_str.style,
            ),
            FieldType::Obj => (
                self.cfg.types.obj.clone(),
                self.cfg.colors.tree.type_obj.style,
            ),
            FieldType::Arr => (
                self.cfg.types.arr.clone(),
                self.cfg.colors.tree.type_arr.style,
            ),
        };
        let line = Line::from(vec![
            Span::styled(name, self.cfg.colors.tree.name.style),
            Span::raw(" "),
            Span::styled(type_str, type_style),
            Span::raw(" "),
            Span::styled(description, self.cfg.colors.tree.value.style),
        ]);
        Text::from(line)
    }
}

impl Data {
    pub fn render(&self, cfg: &Config) -> Text {
        match &self.display {
            Display::Highlight(tokens) => SyntaxToken::render(cfg, tokens),
            Display::Raw(text) => Text::from(text.as_ref()),
        }
    }

    fn raw(text: Cow<'static, str>) -> Self {
        let lines: Vec<_> = text.lines().collect();
        let long_line = lines.iter().max_by_key(|line| line.len());
        let rows = lines.len();
        let columns = long_line.map_or(0, |line| line.len());
        Self {
            display: Display::Raw(text),
            rows,
            columns,
        }
    }

    fn highlight(tokens: Vec<SyntaxToken>) -> Self {
        let (rows, columns) = SyntaxToken::get_size(&tokens);
        Self {
            display: Display::Highlight(tokens),
            rows,
            columns,
        }
    }

    fn null(cfg: &Config) -> Self {
        if cfg.data.disable_highlight {
            Self::raw(Cow::Borrowed(""))
        } else {
            Self::highlight(vec![SyntaxToken::Null("null")])
        }
    }

    fn string(cfg: &Config, s: String) -> Self {
        if cfg.data.disable_highlight {
            Self::raw(Cow::Owned(s))
        } else {
            let lines = s.lines().collect::<Vec<_>>();
            if lines.len() > 1 {
                let mut tokens = Vec::with_capacity(lines.len() * 2);
                for (idx, line) in lines.iter().enumerate() {
                    tokens.push(SyntaxToken::String(line.to_string()));
                    if idx != lines.len() - 1 {
                        tokens.push(SyntaxToken::Break);
                    }
                }
                return Self::highlight(tokens);
            }

            Self::highlight(vec![SyntaxToken::String(s)])
        }
    }

    fn number(cfg: &Config, num: String) -> Self {
        if cfg.data.disable_highlight {
            Self::raw(Cow::Owned(num))
        } else {
            Self::highlight(vec![SyntaxToken::Number(num)])
        }
    }

    fn bool(cfg: &Config, b: bool) -> Self {
        let b = if b { "true" } else { "false" };
        if cfg.data.disable_highlight {
            Self::raw(Cow::Borrowed(b))
        } else {
            Self::highlight(vec![SyntaxToken::Bool(b)])
        }
    }
}
