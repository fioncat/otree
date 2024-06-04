use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use anyhow::Result;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Widget;
use serde_json::Value;
use tui_tree_widget::{Node as TreeNode, TreeData};

use crate::config::Config;
use crate::parse::{ContentType, Parser};

pub struct Tree<'a> {
    pub parser: Rc<Box<dyn Parser>>,

    pub items: Vec<Rc<TreeItem>>,
    pub items_map: HashMap<String, Rc<TreeItem>>,

    cfg: &'a Config,
}

pub struct TreeItem {
    pub name: String,
    pub value: Value,

    pub text: Text<'static>,

    // TODO: Replace this to tokens to implement syntax highlighting.
    pub display: Cow<'static, str>,

    pub children: Vec<Rc<TreeItem>>,
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
            items_map: HashMap::new(),
            cfg,
        };

        // The root value needs to be expanded directly, since we don't want to see a
        // `root` item in the tree.
        let items: Vec<Rc<TreeItem>> = match value {
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

    pub fn get_item(&self, path: &str) -> Option<Rc<TreeItem>> {
        self.items_map.get(path).cloned()
    }

    pub fn get_parser(&self) -> Rc<Box<dyn Parser>> {
        Rc::clone(&self.parser)
    }

    fn build_item(&mut self, parent: Vec<String>, name: String, value: Value) -> Rc<TreeItem> {
        let path = if parent.is_empty() {
            name.clone()
        } else {
            format!("{}/{name}", parent.join("/"))
        };

        let raw_value = value.clone();
        let raw_name = name.clone();
        let item = match value {
            Value::Null => TreeItem {
                name: raw_name,
                value: raw_value,
                text: self.build_item_text(name, FieldType::Null, Cow::Borrowed("null")),
                display: Cow::Borrowed(""),
                children: vec![],
            },
            Value::String(s) => {
                let description = format!("= {s:?}");
                let text = self.build_item_text(name, FieldType::Str, Cow::Owned(description));
                TreeItem {
                    name: raw_name,
                    value: raw_value,
                    text,
                    display: Cow::Owned(s),
                    children: vec![],
                }
            }
            Value::Number(num) => {
                let description = format!("= {num}");
                let text = self.build_item_text(name, FieldType::Num, Cow::Owned(description));
                TreeItem {
                    name: raw_name,
                    value: raw_value,
                    text,
                    display: Cow::Owned(num.to_string()),
                    children: vec![],
                }
            }
            Value::Bool(b) => {
                let description = if b { "= true" } else { "= false" };
                let text = self.build_item_text(name, FieldType::Bool, Cow::Borrowed(description));
                let display = if b { "true" } else { "false" };
                TreeItem {
                    name: raw_name,
                    value: raw_value,
                    text,
                    display: Cow::Borrowed(display),
                    children: vec![],
                }
            }
            Value::Array(arr) => {
                let description = format!(
                    "[ {} {} ]",
                    arr.len(),
                    if arr.len() > 1 { "items" } else { "item" }
                );
                let text = self.build_item_text(name, FieldType::Arr, Cow::Owned(description));
                let display = self.parser.syntax_highlight(&raw_value);

                let mut children = Vec::with_capacity(arr.len());
                for (idx, item) in arr.into_iter().enumerate() {
                    let mut child_parent = parent.to_vec();
                    child_parent.push(raw_name.clone());

                    let child = self.build_item(child_parent, idx.to_string(), item);
                    children.push(child);
                }

                TreeItem {
                    name: raw_name,
                    value: raw_value,
                    text,
                    display: Cow::Owned(display),
                    children,
                }
            }
            Value::Object(obj) => {
                let description = format!(
                    "{{ {} {} }}",
                    obj.len(),
                    if obj.len() > 1 { "fields" } else { "field" }
                );
                let text = self.build_item_text(name, FieldType::Obj, Cow::Owned(description));
                let display = self.parser.syntax_highlight(&raw_value);

                let mut children = Vec::with_capacity(obj.len());
                for (field, item) in obj {
                    let mut child_parent = parent.to_vec();
                    child_parent.push(raw_name.clone());

                    let child = self.build_item(child_parent, field, item);
                    children.push(child);
                }
                TreeItem {
                    name: raw_name,
                    value: raw_value,
                    text,
                    display: Cow::Owned(display),
                    children,
                }
            }
        };

        let item = Rc::new(item);
        self.items_map.insert(path, Rc::clone(&item));
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
                self.cfg.colors.item.type_null.style,
            ),
            FieldType::Num => (
                self.cfg.types.num.clone(),
                self.cfg.colors.item.type_num.style,
            ),
            FieldType::Bool => (
                self.cfg.types.bool.clone(),
                self.cfg.colors.item.type_bool.style,
            ),
            FieldType::Str => (
                self.cfg.types.str.clone(),
                self.cfg.colors.item.type_str.style,
            ),
            FieldType::Obj => (
                self.cfg.types.obj.clone(),
                self.cfg.colors.item.type_obj.style,
            ),
            FieldType::Arr => (
                self.cfg.types.arr.clone(),
                self.cfg.colors.item.type_arr.style,
            ),
        };
        let line = Line::from(vec![
            Span::styled(name, self.cfg.colors.item.name.style),
            Span::raw(" "),
            Span::styled(type_str, type_style),
            Span::raw(" "),
            Span::styled(description, self.cfg.colors.item.description.style),
        ]);
        Text::from(line)
    }

    // From: <https://github.com/EdJoPaTo/tui-rs-tree-widget/blob/main/src/flatten.rs>
    fn flatten(
        open_identifiers: &HashSet<Vec<String>>,
        items: &[Rc<TreeItem>],
        current: &[String],
    ) -> Vec<TreeNode<Vec<String>>> {
        let mut nodes = Vec::new();
        for item in items {
            let mut child_identifier = current.to_vec();
            child_identifier.push(item.name.clone());

            let child_result = open_identifiers
                .contains(&child_identifier)
                .then(|| Self::flatten(open_identifiers, &item.children, &child_identifier));

            nodes.push(TreeNode {
                depth: child_identifier.len() - 1,
                has_children: !item.children.is_empty(),
                height: item.text.height(),
                identifier: child_identifier,
            });

            if let Some(mut child_node) = child_result {
                nodes.append(&mut child_node);
            }
        }
        nodes
    }
}

impl<'a> TreeData for Tree<'a> {
    type Identifier = Vec<String>;

    fn get_nodes(
        &self,
        open_identifiers: &HashSet<Self::Identifier>,
    ) -> Vec<TreeNode<Self::Identifier>> {
        Self::flatten(open_identifiers, &self.items, &[])
    }

    fn render(&self, identifier: &Self::Identifier, area: Rect, buffer: &mut Buffer) {
        let path = identifier.join("/");
        if let Some(item) = self.items_map.get(&path) {
            // TODO: When in search mode, highlight search keyword
            Widget::render(&item.text, area, buffer);
        }
    }
}
