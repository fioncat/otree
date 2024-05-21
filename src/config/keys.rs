use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use crossterm::event::KeyCode;
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
                        (parse_keys(&self.$field, &mut unique).with_context(|| format!("parse key {}", stringify!($field)))?, Action::$value),
                    )+
                ];
                Ok(())
            }
        }
    };
}

fn parse_keys(keys: &[String], unique: &mut HashSet<String>) -> Result<Vec<KeyCode>> {
    let mut codes = Vec::with_capacity(keys.len());
    for key in keys {
        if unique.contains(key) {
            bail!("the key '{key}' is used by another action, cannot be used twice");
        }
        unique.insert(key.clone());

        if !key.starts_with('<') {
            if key.len() != 1 {
                bail!("key length should be equal to 1");
            }
            let char = key.chars().next().unwrap();
            let code = KeyCode::Char(char);
            codes.push(code);
            continue;
        }

        let key = key.replace(['-', '_'], "");

        let code = match key.as_str() {
            "<backspace>" => KeyCode::Backspace,
            "<enter>" => KeyCode::Enter,
            "<left>" => KeyCode::Left,
            "<right>" => KeyCode::Right,
            "<up>" => KeyCode::Up,
            "<down>" => KeyCode::Down,
            "<pageup>" => KeyCode::PageUp,
            "<pagedown>" => KeyCode::PageDown,
            "<tab>" => KeyCode::Tab,
            "<esc>" => KeyCode::Esc,
            _ => bail!("unsupported key: '{}'", key),
        };
        codes.push(code);
    }
    Ok(codes)
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

    #[serde(default = "Keys::default_quit")]
    pub quit: Vec<String>,

    #[serde(skip)]
    pub actions: Vec<(Vec<KeyCode>, Action)>,
}

generate_keys_default!(
    move_up => ["k", "<up>"],
    move_down => ["j", "<down>"],
    move_left => ["h", "<left>"],
    move_right => ["l", "<right>"],
    select_focus => ["<enter>"],
    select_parent => ["p"],
    select_first => ["g"],
    select_last => ["G"],
    close_parent => ["<backspace>"],
    change_root => ["r"],
    reset => ["<esc>"],
    page_up => ["<page-up>", "u"],
    page_down => ["<page-down>", "d"],
    change_layout => ["v"],
    tree_scale_up => ["["],
    tree_scale_down => ["]"],
    switch => ["<tab>"],
    quit => ["q"]
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
    quit => Quit
);

impl Keys {
    pub fn get_key_action(&self, key_code: KeyCode) -> Option<Action> {
        for (codes, action) in self.actions.iter() {
            for code in codes {
                if *code == key_code {
                    return Some(*action);
                }
            }
        }
        None
    }
}
