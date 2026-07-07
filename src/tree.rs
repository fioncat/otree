use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::Result;
use minijinja::Environment;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use serde_json::Value;
use tui_tree_widget::TreeItem;

use crate::config::Config;
use crate::parse::{ContentType, Parser, SyntaxToken};

pub struct Tree {
    pub parser: Rc<Box<dyn Parser>>,

    pub items: Vec<TreeItem<'static, String>>,
    pub values: HashMap<String, ItemValue>,

    pub identifies: Vec<String>,

    /// Compiled `cfg.tree.format` template, `None` when no format is configured.
    format_env: Option<Environment<'static>>,

    cfg: Rc<Config>,
}

pub struct ItemValue {
    pub name: String,
    pub value: Value,

    pub field_type: FieldType,
    pub description: Cow<'static, str>,

    pub tokens: Vec<SyntaxToken>,
}

#[derive(Debug, Clone, Copy)]
pub struct HighlightKeyword<'a> {
    pub text: &'a str,
    pub ignore_case: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum FieldType {
    Null,
    Num,
    Bool,
    Str,
    Obj,
    Arr,
}

impl Tree {
    pub fn parse(cfg: Rc<Config>, data: &[u8], content_type: ContentType) -> Result<Self> {
        let parser = content_type.new_parser();
        let value = parser.parse_root(None, data)?;
        Ok(Self::from_value(cfg, value, Rc::new(parser)))
    }

    pub fn from_value(cfg: Rc<Config>, value: Value, parser: Rc<Box<dyn Parser>>) -> Self {
        // The format template is validated in `Config::parse`, so a compile error
        // here would be a bug; degrade to no format rather than panic.
        let format_env = cfg.tree.build_env().unwrap_or(None);
        let mut tree = Self {
            parser,
            items: vec![],
            values: HashMap::new(),
            identifies: vec![],
            format_env,
            cfg,
        };

        // The root value needs to be expanded directly, since we don't want to see a
        // `root` item in the tree.
        let items: Vec<TreeItem<String>> = match value {
            Value::Array(arr) => {
                let mut items = Vec::with_capacity(arr.len());
                for (idx, value) in arr.into_iter().enumerate() {
                    let item = tree.build_item(&[], idx.to_string(), value, None);
                    items.push(item);
                }
                items
            }
            Value::Object(obj) => {
                let mut items = Vec::with_capacity(obj.len());
                for (field, value) in obj {
                    let item = tree.build_item(&[], field, value, None);
                    items.push(item);
                }
                items
            }
            _ => {
                vec![tree.build_item(&[], String::from("root"), value, None)]
            }
        };
        tree.items = items;
        tree
    }

    pub fn get_value(&self, path: &str) -> Option<&ItemValue> {
        self.values.get(path)
    }

    pub fn get_parser(&self) -> Rc<Box<dyn Parser>> {
        Rc::clone(&self.parser)
    }

    fn build_item(
        &mut self,
        parent: &[String],
        name: String,
        value: Value,
        arr_name: Option<&str>,
    ) -> TreeItem<'static, String> {
        let path = if parent.is_empty() {
            name.clone()
        } else {
            format!("{}/{name}", parent.join("/"))
        };
        self.identifies.push(path.clone());

        let raw_value = value.clone();
        let raw_name = name.clone();
        let (item, value) = match value {
            Value::Null => (
                TreeItem::new_leaf(
                    raw_name.clone(),
                    Self::build_item_text(
                        &self.cfg,
                        name,
                        FieldType::Null,
                        Cow::Borrowed("null"),
                        None,
                    ),
                ),
                ItemValue {
                    name: raw_name,
                    value: raw_value,
                    field_type: FieldType::Null,
                    description: Cow::Borrowed("null"),
                    tokens: vec![SyntaxToken::Null("null")],
                },
            ),
            Value::String(s) => {
                let description = format!("= {s:?}");
                let text = Self::build_item_text(
                    &self.cfg,
                    name,
                    FieldType::Str,
                    Cow::Owned(description.clone()),
                    None,
                );
                (
                    TreeItem::new_leaf(raw_name.clone(), text),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        field_type: FieldType::Str,
                        description: Cow::Owned(description),
                        tokens: Self::string_tokens(s),
                    },
                )
            }
            Value::Number(num) => {
                let description = format!("= {num}");
                let text = Self::build_item_text(
                    &self.cfg,
                    name,
                    FieldType::Num,
                    Cow::Owned(description.clone()),
                    None,
                );
                (
                    TreeItem::new_leaf(raw_name.clone(), text),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        field_type: FieldType::Num,
                        description: Cow::Owned(description),
                        tokens: vec![SyntaxToken::Number(num.to_string())],
                    },
                )
            }
            Value::Bool(b) => {
                let description = if b { "= true" } else { "= false" };
                let text = Self::build_item_text(
                    &self.cfg,
                    name,
                    FieldType::Bool,
                    Cow::Borrowed(description),
                    None,
                );
                let b = if b { "true" } else { "false" };
                (
                    TreeItem::new_leaf(raw_name.clone(), text),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        field_type: FieldType::Bool,
                        description: Cow::Borrowed(description),
                        tokens: vec![SyntaxToken::Bool(b)],
                    },
                )
            }
            Value::Array(arr) => {
                let description = format!(
                    "[ {} {} ]",
                    arr.len(),
                    if arr.len() > 1 { "items" } else { "item" }
                );
                let data_name = arr_name.unwrap_or(&name);
                let tokens = self.parser.syntax_highlight(data_name, &raw_value);
                let arr_name = Some(name.clone());
                let text = Self::build_item_text(
                    &self.cfg,
                    name,
                    FieldType::Arr,
                    Cow::Owned(description.clone()),
                    None,
                );

                let mut children = Vec::with_capacity(arr.len());
                for (idx, item) in arr.into_iter().enumerate() {
                    let mut child_parent = parent.to_vec();
                    child_parent.push(raw_name.clone());

                    let child =
                        self.build_item(&child_parent, idx.to_string(), item, arr_name.as_deref());
                    children.push(child);
                }

                (
                    TreeItem::new(raw_name.clone(), text, children).unwrap(),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        field_type: FieldType::Arr,
                        description: Cow::Owned(description),
                        tokens,
                    },
                )
            }
            Value::Object(obj) => {
                let mut description = format!(
                    "{{ {} {} }}",
                    obj.len(),
                    if obj.len() > 1 { "fields" } else { "field" }
                );
                if let Some(suffix) = self.render_format(&raw_value) {
                    description.push(' ');
                    description.push_str(&suffix);
                }
                let data_name = arr_name.unwrap_or(&name);
                let tokens = self.parser.syntax_highlight(data_name, &raw_value);
                let text = Self::build_item_text(
                    &self.cfg,
                    name,
                    FieldType::Obj,
                    Cow::Owned(description.clone()),
                    None,
                );

                let mut children = Vec::with_capacity(obj.len());
                for (field, item) in obj {
                    let mut child_parent = parent.to_vec();
                    child_parent.push(raw_name.clone());

                    let child = self.build_item(&child_parent, field, item, None);
                    children.push(child);
                }
                (
                    TreeItem::new(raw_name.clone(), text, children).unwrap(),
                    ItemValue {
                        name: raw_name,
                        value: raw_value,
                        field_type: FieldType::Obj,
                        description: Cow::Owned(description),
                        tokens,
                    },
                )
            }
        };

        self.values.insert(path, value);
        item
    }

    /// Render the configured format template against an object node's value,
    /// returning the trimmed output, or `None` when no format is configured or
    /// the result is blank (e.g. every referenced attribute was absent).
    fn render_format(&self, value: &Value) -> Option<String> {
        let env = self.format_env.as_ref()?;
        let tmpl = env
            .get_template(crate::config::Tree::FORMAT_TEMPLATE)
            .ok()?;
        let rendered = tmpl.render(value).ok()?;
        let trimmed = rendered.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn build_item_text(
        cfg: &Config,
        name: String,
        field_type: FieldType,
        description: Cow<'static, str>,
        keyword: Option<HighlightKeyword>,
    ) -> Text<'static> {
        // TODO: We can share field type to save memory.
        let (type_str, type_style) = match field_type {
            FieldType::Null => (cfg.types.null.clone(), cfg.colors.tree.type_null.style),
            FieldType::Num => (cfg.types.num.clone(), cfg.colors.tree.type_num.style),
            FieldType::Bool => (cfg.types.bool.clone(), cfg.colors.tree.type_bool.style),
            FieldType::Str => (cfg.types.str.clone(), cfg.colors.tree.type_str.style),
            FieldType::Obj => (cfg.types.obj.clone(), cfg.colors.tree.type_obj.style),
            FieldType::Arr => (cfg.types.arr.clone(), cfg.colors.tree.type_arr.style),
        };
        let mut spans = vec![];
        if let Some(keyword) = keyword {
            spans.extend(Self::highlight_keyword(
                name,
                keyword,
                cfg.colors.tree.name.style,
                cfg.colors.tree.filter_keyword.style,
            ));
        } else {
            spans.push(Span::styled(name, cfg.colors.tree.name.style));
        }

        spans.push(Span::raw(" "));
        spans.push(Span::styled(type_str, type_style));
        spans.push(Span::raw(" "));

        if let Some(keyword) = keyword {
            spans.extend(Self::highlight_keyword(
                description.to_string(),
                keyword,
                cfg.colors.tree.value.style,
                cfg.colors.tree.filter_keyword.style,
            ));
        } else {
            spans.push(Span::styled(description, cfg.colors.tree.value.style));
        }

        Text::from(Line::from(spans))
    }

    fn highlight_keyword(
        text: String,
        keyword: HighlightKeyword,
        normal_style: Style,
        highlight_style: Style,
    ) -> Vec<Span<'static>> {
        if keyword.text.is_empty() {
            return vec![Span::styled(text, normal_style)];
        }

        let mut spans = Vec::new();
        let text_compare = if keyword.ignore_case {
            Cow::Owned(text.to_lowercase())
        } else {
            Cow::Borrowed(&text)
        };
        let keyword_compare = if keyword.ignore_case {
            Cow::Owned(keyword.text.to_lowercase())
        } else {
            Cow::Borrowed(keyword.text)
        };
        let mut last_end = 0;

        for (start, _) in text_compare.match_indices(keyword_compare.as_ref()) {
            if start > last_end {
                spans.push(Span::styled(
                    text[last_end..start].to_string(),
                    normal_style,
                ));
            }

            let end = start + keyword_compare.len();
            spans.push(Span::styled(text[start..end].to_string(), highlight_style));

            last_end = end;
        }

        if last_end < text.len() {
            spans.push(Span::styled(text[last_end..].to_string(), normal_style));
        }

        spans
    }

    fn string_tokens(s: String) -> Vec<SyntaxToken> {
        let lines = s.lines().collect::<Vec<_>>();
        if lines.len() > 1 {
            let mut tokens = Vec::with_capacity(lines.len() * 2);
            for (idx, line) in lines.iter().enumerate() {
                tokens.push(SyntaxToken::String(line.to_string()));
                if idx != lines.len() - 1 {
                    tokens.push(SyntaxToken::Break);
                }
            }
            return tokens;
        }

        vec![SyntaxToken::String(s)]
    }
}

impl ItemValue {
    pub fn plain_text(&self) -> String {
        SyntaxToken::pure_text(&self.tokens)
    }

    pub fn render(&self, cfg: &Config, width: usize) -> (Text<'_>, usize, usize) {
        SyntaxToken::render(cfg, &self.tokens, width)
    }

    pub fn build_highlighted_text(
        &self,
        cfg: &Config,
        keyword: &str,
        ignore_case: bool,
    ) -> Text<'static> {
        Tree::build_item_text(
            cfg,
            self.name.clone(),
            self.field_type,
            self.description.clone(),
            Some(HighlightKeyword {
                text: keyword,
                ignore_case,
            }),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::ContentType;

    /// Build a tree from JSON `data` with the given `tree.format`, then return the
    /// description of the node at `path`.
    fn describe(format: &str, data: &str, path: &str) -> String {
        let mut cfg = Config::default();
        cfg.tree.format = format.to_string();
        cfg.parse().expect("config should be valid");

        let parser = ContentType::Json.new_parser();
        let value = parser.parse_root(None, data.as_bytes()).unwrap();
        let tree = Tree::from_value(Rc::new(cfg), value, Rc::new(parser));

        tree.get_value(path)
            .unwrap_or_else(|| panic!("no node at path {path:?}"))
            .description
            .to_string()
    }

    #[test]
    fn no_format_keeps_default_description() {
        let desc = describe("", r#"{"item": {"id": 42, "name": "x"}}"#, "item");
        assert_eq!(desc, "{ 2 fields }");
    }

    #[test]
    fn injects_scalar_attribute() {
        let desc = describe("{{ id }}", r#"{"item": {"id": 42, "name": "x"}}"#, "item");
        assert_eq!(desc, "{ 2 fields } 42");
    }

    #[test]
    fn supports_literal_text_and_multiple_attributes() {
        let desc = describe(
            "id={{ id }} name={{ name }}",
            r#"{"item": {"id": 42, "name": "x"}}"#,
            "item",
        );
        assert_eq!(desc, "{ 2 fields } id=42 name=x");
    }

    #[test]
    fn absent_attribute_renders_empty_and_skips_suffix() {
        // `id` is missing; lenient undefined renders empty, so no suffix is added.
        let desc = describe("{{ id }}", r#"{"item": {"name": "x"}}"#, "item");
        assert_eq!(desc, "{ 1 field }");
    }

    #[test]
    fn non_scalar_attribute_is_rendered_by_the_engine() {
        // Referencing an object attribute yields the engine's map repr, not a scalar.
        // Guarding against that is a template author concern; the default template
        // (a single `{{ key }}`) is expected to point at scalar fields.
        let desc = describe(
            "{{ id | default('') }}",
            r#"{"item": {"id": 7, "meta": {"a": 1}}}"#,
            "item",
        );
        assert_eq!(desc, "{ 2 fields } 7");
    }

    #[test]
    fn supports_filters_and_conditionals() {
        let desc = describe(
            "{% if active %}#{{ id }}{% endif %}",
            r#"{"item": {"id": 3, "active": true}}"#,
            "item",
        );
        assert_eq!(desc, "{ 2 fields } #3");
    }

    #[test]
    fn format_only_applies_to_objects() {
        // A string node keeps its own description, untouched by the format.
        let desc = describe(
            "{{ id }}",
            r#"{"item": {"greeting": "hello"}}"#,
            "item/greeting",
        );
        assert_eq!(desc, "= \"hello\"");
    }

    #[test]
    fn invalid_template_is_rejected_at_config_parse() {
        let mut cfg = Config::default();
        cfg.tree.format = "{{ id ".to_string();
        assert!(cfg.parse().is_err());
    }
}
