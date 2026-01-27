use std::io::BufRead;
use std::mem::take;

use anyhow::Result;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::{to_value, Map, Value};

use super::{Parser, SyntaxToken};

pub struct XmlParser {}

impl Parser for XmlParser {
    fn extension(&self) -> &'static str {
        "xml"
    }

    fn allow_array_root(&self) -> bool {
        false
    }

    fn parse(&self, data: &str) -> Result<Value> {
        let mut reader = Reader::from_str(data);

        let config = reader.config_mut();
        config.expand_empty_elements = true;

        read(&mut reader, 0)
    }

    fn syntax_highlight(&self, name: &str, value: &Value) -> Vec<SyntaxToken> {
        highlight(name, value.clone(), 0)
    }
}

trait AttrMap {
    fn insert_text(&mut self, value: &Value) -> Option<Value>;
    fn insert_text_node(&mut self, value: Value);
}

impl AttrMap for Map<String, Value> {
    fn insert_text(&mut self, value: &Value) -> Option<Value> {
        if !self.is_empty() {
            if value.is_string() {
                self.insert_text_node(value.clone());
            }
            if let Ok(attrs) = to_value(take(self)) {
                return Some(attrs);
            }
        }
        None
    }

    fn insert_text_node(&mut self, value: Value) {
        self.insert("#text".to_string(), value);
    }
}

#[derive(Default)]
struct NodeValues {
    node: Map<String, Value>,
    nodes: Vec<Map<String, Value>>,
    nodes_are_map: Vec<bool>,
    values: Vec<Value>,
}

impl NodeValues {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, key: String, value: Value) {
        self.node.insert(key, value);
    }

    fn insert_cdata(&mut self, value: &str) {
        let key = "#cdata".to_string();
        let new_value = match self.node.get(&key) {
            Some(existing) => {
                let mut old_value = existing.as_str().unwrap().to_string();
                old_value.push_str(value);
                old_value
            }
            None => value.to_string(),
        };
        self.node.insert(key, Value::String(new_value));
    }

    fn insert_text(&mut self, text: &str) {
        if self.node.is_empty() {
            // if directly preceded by another string, append to it
            if let Some(value) = self.values.pop() {
                let mut value_text = value.as_str().unwrap_or_default().to_string();
                value_text.push_str(text);
                self.values.push(Value::String(value_text));
                return;
            }
        } else {
            // don't insert whitespace between nodes
            if text.trim().is_empty() {
                return;
            }

            self.nodes.push(take(&mut self.node));
            self.nodes_are_map.push(true);
        }

        self.values.push(Value::String(text.to_string()));
        self.nodes_are_map.push(false);
    }

    fn remove_entry(&mut self, key: &String) -> Option<Value> {
        if self.node.contains_key(key) {
            if let Some((_, existing)) = self.node.remove_entry(key) {
                return Some(existing);
            }
        }
        None
    }

    fn get_value(&mut self) -> Value {
        if !self.node.is_empty() {
            self.nodes.push(take(&mut self.node));
            self.nodes_are_map.push(true);
        }

        if !self.nodes.is_empty() {
            // If we had collected some non-whitespace text along the way, that
            // needs to be inserted so we don't lose it

            if self.nodes.len() == 1 && self.values.len() <= 1 {
                if self.values.len() == 1 {
                    let value = self.values.remove(0);
                    let text = value.as_str().unwrap_or_default().trim();
                    if !text.is_empty() {
                        self.nodes[0].insert_text_node(Value::String(text.to_string()));
                    }
                }
                return to_value(&self.nodes[0]).expect("Failed to #to_value() a node!");
            }
            for (index, node_is_map) in self.nodes_are_map.iter().enumerate() {
                if *node_is_map {
                    self.values
                        .insert(index, Value::Object(self.nodes.remove(0)));
                }
            }
        }

        // trim any values left, removing empty strings
        self.values = self
            .values
            .drain(..)
            .filter_map(|mut value| {
                if let Value::String(text) = value {
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    value = Value::String(trimmed.to_string());
                }

                Some(value)
            })
            .collect();

        match self.values.len() {
            0 => Value::Null,
            1 => self.values.pop().unwrap(),
            _ => Value::Array(take(&mut self.values)),
        }
    }
}

#[expect(clippy::only_used_in_recursion)] // want to use depth at some point
fn read<R: BufRead>(reader: &mut Reader<R>, depth: u64) -> Result<Value> {
    let mut buf = Vec::new();
    let mut nodes = NodeValues::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let name = String::from_utf8(e.name().into_inner().to_vec())?;
                let mut child = read(reader, depth + 1)?;
                let mut attrs: Map<_, _> = e
                    .attributes()
                    .map(|attr| {
                        let attr = attr?;
                        let (key, value) = (
                            String::from_utf8(attr.key.into_inner().to_vec())?,
                            String::from_utf8(attr.value.to_vec())?,
                        );

                        Ok((format!("@{key}"), Value::String(value)))
                    })
                    .collect::<Result<_>>()?;

                if child.is_null() && !attrs.is_empty() {
                    // Handle empty node with fields
                    // Like: <node field1="value1" field2="value2"/>
                    child = Value::Object(Map::new());
                }

                // If the child is already an object, that's where attributes should end up in
                if child.is_object() {
                    // We want to have them at the start though, while still being listed
                    // Since serde_json::Map doesn't really expose that much of indexmap::Map,
                    // we'll just hack up our own semi-splice *sigh*
                    let child = child.as_object_mut().unwrap();
                    *child = take(&mut attrs).into_iter().chain(take(child)).collect();
                }

                if let Some(mut existing) = nodes.remove_entry(&name) {
                    let mut ents = Vec::new();
                    if let Value::Array(ref mut existing) = existing {
                        ents.append(existing);
                    } else {
                        ents.push(existing);
                    }

                    attrs.insert_text(&child);
                    ents.push(child);

                    nodes.insert(name, Value::Array(ents));
                } else if let Some(attrs) = attrs.insert_text(&child) {
                    nodes.insert(name, attrs);
                } else {
                    nodes.insert(name, child);
                }
            }
            Event::Text(ref e) => {
                let decoded = e.decode()?;
                nodes.insert_text(&decoded);
            }
            Event::CData(ref e) => {
                let decoded = e.decode()?;
                nodes.insert_cdata(&decoded);
            }
            Event::GeneralRef(ref e) => {
                if let Some(ch) = e.resolve_char_ref()? {
                    nodes.insert_text(&ch.to_string());
                } else {
                    let decoded = e.decode()?;
                    if let Some(ent) = resolve_predefined_entity(&decoded) {
                        nodes.insert_text(ent);
                    }
                }
            }
            Event::End(ref _e) => break,
            Event::Eof => break,
            _ => (),
        }
    }

    Ok(nodes.get_value())
}

fn highlight(name: &str, value: Value, indent: usize) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    if let Value::Object(obj) = value {
        let mut attrs = Vec::new();
        let mut text = None;
        let mut child_obj = Map::new();

        for (child_key, child_value) in obj {
            if child_key.starts_with('@') {
                let attr = child_key.trim_start_matches('@').to_string();
                let Value::String(value) = child_value.clone() else {
                    unreachable!("xml parser should only return string attributes")
                };
                attrs.push((attr, value));
                continue;
            }

            if child_key == "#text" {
                let Value::String(inner) = child_value.clone() else {
                    unreachable!("xml parser should only return string text nodes")
                };
                text = Some(inner);
                continue;
            }

            child_obj.insert(child_key, child_value);
        }

        if !name.is_empty() {
            tokens.push(SyntaxToken::Indent(indent));
            if attrs.is_empty() {
                tokens.push(SyntaxToken::Tag(format!("<{name}>")));
            } else {
                tokens.push(SyntaxToken::Tag(format!("<{name}")));
                for (attr, value) in attrs {
                    tokens.push(SyntaxToken::Symbol(" "));
                    tokens.push(SyntaxToken::Name(attr));
                    tokens.push(SyntaxToken::Symbol("="));
                    tokens.push(SyntaxToken::String(format!("{value:?}")));
                }
                tokens.push(SyntaxToken::Tag(String::from(">")));
            }
        }

        if let Some(text) = text {
            tokens.push(SyntaxToken::String(text));
        } else if !child_obj.is_empty() {
            if !name.is_empty() {
                tokens.push(SyntaxToken::Break);
            }
            for (child_key, child_value) in child_obj {
                let child_indent = if name.is_empty() { indent } else { indent + 1 };
                let child_tokens = highlight(&child_key, child_value, child_indent);
                tokens.extend(child_tokens);
            }
            if !name.is_empty() {
                tokens.push(SyntaxToken::Indent(indent));
            }
        }

        if !name.is_empty() {
            tokens.push(SyntaxToken::Tag(format!("</{name}>")));
            tokens.push(SyntaxToken::Break);
        }

        return tokens;
    }

    match value {
        Value::String(s) => {
            tokens.push(SyntaxToken::Indent(indent));
            tokens.push(SyntaxToken::Tag(format!("<{name}>")));
            tokens.push(SyntaxToken::String(s));
            tokens.push(SyntaxToken::Tag(format!("</{name}>")));
            tokens.push(SyntaxToken::Break);
        }
        Value::Array(arr) => {
            for child_item in arr {
                let child_tokens = highlight(name, child_item, indent);
                tokens.extend(child_tokens);
            }
        }
        Value::Null => {
            tokens.push(SyntaxToken::Indent(indent));
            tokens.push(SyntaxToken::Tag(format!("<{name}></{name}>")));
            tokens.push(SyntaxToken::Break);
        }
        _ => {}
    }

    tokens
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_syntax_highlight() {
        let test_cases = [
            (
                include_str!("test_cases/xml/object.json"),
                include_str!("test_cases/xml/object.xml"),
            ),
            (
                include_str!("test_cases/xml/array.json"),
                include_str!("test_cases/xml/array.xml"),
            ),
            (
                include_str!("test_cases/xml/array_of_objects.json"),
                include_str!("test_cases/xml/array_of_objects.xml"),
            ),
            (
                include_str!("test_cases/xml/attrs.json"),
                include_str!("test_cases/xml/attrs.xml"),
            ),
        ];
        let parser = XmlParser {};
        for (raw, expect) in test_cases {
            let value: Value = serde_json::from_str(raw).unwrap();
            let tokens = parser.syntax_highlight("", &value);
            let result = SyntaxToken::pure_text(&tokens);
            assert_eq!(result, expect);
        }
    }

    #[test]
    fn test_parse() {
        let test_cases = [
            (
                include_str!("test_cases/xml/object.xml"),
                include_str!("test_cases/xml/object.json"),
            ),
            (
                include_str!("test_cases/xml/array.xml"),
                include_str!("test_cases/xml/array.json"),
            ),
            (
                include_str!("test_cases/xml/array_of_objects.xml"),
                include_str!("test_cases/xml/array_of_objects.json"),
            ),
            (
                include_str!("test_cases/xml/attrs.xml"),
                include_str!("test_cases/xml/attrs.json"),
            ),
        ];

        let parser = XmlParser {};
        for (xml_data, json_data) in test_cases {
            let value = parser.parse(xml_data).unwrap();
            let expected: Value = serde_json::from_str(json_data).unwrap();
            assert_eq!(value, expected);
        }
    }
}
