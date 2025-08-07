use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

macro_rules! generate_keys_default {
    ($($field:ident => $value:expr),+) => {
        impl Keys {
            ::paste::paste! {
                pub fn default() -> Self {
                    Self {
                        $(
                            $field: Self::[<default_ $field>](),
                        )+
                        actions: vec![],
                    }
                }
            }

            $(
                ::paste::paste! {
                    pub fn [<default_ $field>]() -> Vec<String> {
                        $value.into_iter().map(|s| s.to_string()).collect()
                    }
                }
            )+
        }
    };
}

macro_rules! generate_actions {
    ($($field:ident => $value:ident),+) => {
        #[derive(Debug, Clone, Copy)]
        pub enum Action {
            $($value),+
        }

        impl Keys {
            pub fn parse(&mut self) -> Result<()> {
                let mut unique = HashSet::new();
                self.actions = vec![
                    $(
                        (Key::parse_keys(&self.$field, &mut unique).with_context(|| format!("parse keys for action {}", stringify!($field)))?, Action::$value),
                    )+
                ];
                Ok(())
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),

    Ctrl(char),
    Alt(char),

    F(u8),

    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Tab,
    Esc,
}

impl Key {
    fn from_event(event: KeyEvent) -> Option<Self> {
        if event.modifiers == KeyModifiers::CONTROL {
            if let KeyCode::Char(char) = event.code {
                return Some(Self::Ctrl(char));
            }

            return None;
        }

        if event.modifiers == KeyModifiers::ALT {
            if let KeyCode::Char(char) = event.code {
                return Some(Self::Alt(char));
            }

            return None;
        }

        if event.modifiers == KeyModifiers::SHIFT {
            if let KeyCode::Char(char) = event.code {
                return Some(Self::Char(char));
            }

            return None;
        }

        if event.modifiers != KeyModifiers::NONE {
            return None;
        }

        match event.code {
            KeyCode::Char(char) => Some(Self::Char(char)),
            KeyCode::Backspace => Some(Self::Backspace),
            KeyCode::Enter => Some(Self::Enter),
            KeyCode::Left => Some(Self::Left),
            KeyCode::Right => Some(Self::Right),
            KeyCode::Up => Some(Self::Up),
            KeyCode::Down => Some(Self::Down),
            KeyCode::PageUp => Some(Self::PageUp),
            KeyCode::PageDown => Some(Self::PageDown),
            KeyCode::Tab => Some(Self::Tab),
            KeyCode::Esc => Some(Self::Esc),
            KeyCode::F(n) => Some(Self::F(n)),
            _ => None,
        }
    }

    fn parse(key: &str) -> Result<Self> {
        if !key.starts_with('<') {
            if key.len() != 1 {
                bail!("invalid key '{key}', length should be equal to 1");
            }
            let char = key.chars().next().unwrap();
            return Ok(Self::Char(char));
        }

        let raw_key = key;

        let key = key.strip_prefix('<').unwrap();
        let key = match key.strip_suffix('>') {
            Some(key) => key,
            None => bail!("invalid key '{raw_key}', should be ends with '>'"),
        };

        if let Some(key) = key.strip_prefix("ctrl-") {
            if key.len() != 1 {
                bail!("invalid key '{raw_key}', should be '<ctrl-x>'");
            }
            let char = key.chars().next().unwrap();
            return Ok(Self::Ctrl(char));
        }

        if let Some(key) = key.strip_prefix("alt-") {
            if key.len() != 1 {
                bail!("invalid key '{raw_key}', should be '<alt-x>'");
            }
            let char = key.chars().next().unwrap();
            return Ok(Self::Alt(char));
        }

        if let Some(key) = key.strip_prefix('f') {
            let n = match key.parse::<u8>() {
                Ok(n) => n,
                Err(_) => bail!("invalid key '{raw_key}', should be '<fN>'"),
            };

            if n == 0 || n > 12 {
                bail!("invalid key '{raw_key}', fN should be in range [1, 12]");
            }

            return Ok(Self::F(n));
        }

        let key = key.replace(['-', '_'], "");
        Ok(match key.as_str() {
            "backspace" => Self::Backspace,
            "enter" => Self::Enter,
            "left" => Self::Left,
            "right" => Self::Right,
            "up" => Self::Up,
            "down" => Self::Down,
            "pageup" => Self::PageUp,
            "pagedown" => Self::PageDown,
            "tab" => Self::Tab,
            "esc" => Self::Esc,
            _ => bail!("unsupported key '{raw_key}'"),
        })
    }

    fn parse_keys(raw_keys: &[String], unique: &mut HashSet<String>) -> Result<Vec<Self>> {
        let mut keys = Vec::with_capacity(raw_keys.len());
        for raw_key in raw_keys {
            if unique.contains(raw_key) {
                bail!("the key '{raw_key}' is used by another action, cannot be used twice");
            }
            unique.insert(raw_key.clone());
            keys.push(Self::parse(raw_key)?);
        }
        Ok(keys)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keys {
    #[serde(default = "Keys::default_move_up")]
    pub move_up: Vec<String>,
    #[serde(default = "Keys::default_move_down")]
    pub move_down: Vec<String>,

    #[serde(default = "Keys::default_move_left")]
    pub move_left: Vec<String>,
    #[serde(default = "Keys::default_move_right")]
    pub move_right: Vec<String>,

    #[serde(default = "Keys::default_select_focus")]
    pub select_focus: Vec<String>,
    #[serde(default = "Keys::default_select_parent")]
    pub select_parent: Vec<String>,
    #[serde(default = "Keys::default_select_first")]
    pub select_first: Vec<String>,
    #[serde(default = "Keys::default_select_last")]
    pub select_last: Vec<String>,

    #[serde(default = "Keys::default_close_parent")]
    pub close_parent: Vec<String>,

    #[serde(default = "Keys::default_change_root")]
    pub change_root: Vec<String>,

    #[serde(default = "Keys::default_reset")]
    pub reset: Vec<String>,

    #[serde(default = "Keys::default_page_up")]
    pub page_up: Vec<String>,
    #[serde(default = "Keys::default_page_down")]
    pub page_down: Vec<String>,

    #[serde(default = "Keys::default_change_layout")]
    pub change_layout: Vec<String>,

    #[serde(default = "Keys::default_tree_scale_up")]
    pub tree_scale_up: Vec<String>,

    #[serde(default = "Keys::default_tree_scale_down")]
    pub tree_scale_down: Vec<String>,

    #[serde(default = "Keys::default_switch")]
    pub switch: Vec<String>,

    #[serde(default = "Keys::default_edit")]
    pub edit: Vec<String>,

    #[serde(default = "Keys::default_copy_name")]
    pub copy_name: Vec<String>,

    #[serde(default = "Keys::default_copy_value")]
    pub copy_value: Vec<String>,

    #[serde(default = "Keys::default_filter")]
    pub filter: Vec<String>,

    #[serde(default = "Keys::default_filter_key")]
    pub filter_key: Vec<String>,

    #[serde(default = "Keys::default_filter_value")]
    pub filter_value: Vec<String>,

    #[serde(default = "Keys::default_filter_switch_ignore_case")]
    pub filter_switch_ignore_case: Vec<String>,

    #[serde(default = "Keys::default_quit")]
    pub quit: Vec<String>,

    #[serde(skip)]
    actions: Vec<(Vec<Key>, Action)>,
}

generate_keys_default!(
    move_up => ["k", "<up>"],
    move_down => ["j", "<down>"],
    move_left => ["h", "<left>"],
    move_right => ["l", "<right>"],
    select_focus => ["<enter>"],
    select_parent => ["p"],
    select_first => ["g", "<ctrl-a>"],
    select_last => ["G", "<ctrl-l>"],
    close_parent => ["<backspace>"],
    change_root => ["r"],
    reset => ["<esc>"],
    page_up => ["<page-up>", "<ctrl-y>"],
    page_down => ["<page-down>", "<ctrl-e>"],
    change_layout => ["v"],
    tree_scale_up => ["["],
    tree_scale_down => ["]"],
    switch => ["<tab>"],
    edit => ["e"],
    copy_name => ["y"],
    copy_value => ["Y"],
    filter => ["/"],
    filter_key => ["?"],
    filter_value => ["*"],
    filter_switch_ignore_case => ["I"],
    quit => ["<ctrl-c>", "q"]
);

generate_actions!(
    move_up => MoveUp,
    move_down => MoveDown,
    move_left => MoveLeft,
    move_right => MoveRight,
    select_focus => SelectFocus,
    select_parent => SelectParent,
    select_first => SelectFirst,
    select_last => SelectLast,
    close_parent => CloseParent,
    change_root => ChangeRoot,
    reset => Reset,
    page_up => PageUp,
    page_down => PageDown,
    change_layout => ChangeLayout,
    tree_scale_up => TreeScaleUp,
    tree_scale_down => TreeScaleDown,
    switch => Switch,
    edit => Edit,
    copy_name => CopyName,
    copy_value => CopyValue,
    filter => Filter,
    filter_key => FilterKey,
    filter_value => FilterValue,
    filter_switch_ignore_case => FilterSwitchIgnoreCase,
    quit => Quit
);

#[derive(Debug, Clone, Copy)]
pub struct KeyAction {
    pub key: Key,
    pub action: Option<Action>,
}

impl Keys {
    pub fn get_key_action(&self, event: KeyEvent) -> Option<KeyAction> {
        let event_key = Key::from_event(event)?;
        let mut current_action = None;
        for (keys, action) in self.actions.iter() {
            for key in keys {
                if *key == event_key {
                    current_action = Some(*action);
                }
            }
        }
        Some(KeyAction {
            key: event_key,
            action: current_action,
        })
    }
}
