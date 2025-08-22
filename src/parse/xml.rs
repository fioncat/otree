use std::io::BufRead;
use std::mem::take;

use anyhow::Result;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::{to_value, Map, Value};

use super::json;
use super::{Parser, SyntaxToken};

pub struct XmlParser {}

impl Parser for XmlParser {
    fn extension(&self) -> &'static str {
        "xml"
    }

    fn parse(&self, data: &str) -> Result<Value> {
        let mut reader = Reader::from_str(data);

        let config = reader.config_mut();
        config.expand_empty_elements = true;

        read(&mut reader, 0)
    }

    fn to_string(&self, value: &Value) -> String {
        serde_json::to_string_pretty(value).expect("serialize JSON")
    }

    fn syntax_highlight(&self, value: &Value) -> Vec<SyntaxToken> {
        json::highlight(value, 0, false)
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

struct NodeValues {
    node: Map<String, Value>,
    nodes: Vec<Map<String, Value>>,
    nodes_are_map: Vec<bool>,
    values: Vec<Value>,
}

impl NodeValues {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            node: Map::new(),
            nodes: Vec::new(),
            nodes_are_map: Vec::new(),
        }
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
            .clone()
            .into_iter()
            .filter_map(|value| {
                if value.is_string() {
                    let trimmed = value.as_str().unwrap_or_default().trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    return Some(Value::String(trimmed.to_string()));
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

fn read<R: BufRead>(reader: &mut Reader<R>, _depth: u64) -> Result<Value> {
    let mut buf = Vec::new();
    let mut nodes = NodeValues::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let name = String::from_utf8(e.name().into_inner().to_vec())?;
                let mut child = read(reader, _depth + 1)?;
                let mut attrs = Map::new();

                for a in e.attributes() {
                    let a = a?;
                    let key = String::from_utf8(a.key.into_inner().to_vec())?;
                    let value = String::from_utf8(a.value.to_vec())?;

                    let key = format!("@{key}");
                    let value = Value::String(value);

                    // If the child is already an object, that's where the insert
                    // should happen
                    if child.is_object() {
                        child.as_object_mut().unwrap().insert(key, value);
                    } else {
                        attrs.insert(key, value);
                    }
                }

                if let Some(mut existing) = nodes.remove_entry(&name) {
                    let mut ents: Vec<Value> = Vec::new();
                    if existing.is_array() {
                        let existing = existing.as_array_mut().unwrap();
                        while !existing.is_empty() {
                            ents.push(existing.remove(0));
                        }
                    } else {
                        ents.push(existing);
                    }

                    // nodes with attributes need to be handled special
                    if let Some(attrs) = attrs.insert_text(&child) {
                        ents.push(attrs);
                    } else {
                        ents.push(child);
                    }

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
