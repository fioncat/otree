#![allow(dead_code)]

mod config;
mod interactive;
mod tree;

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::interactive::app::{App, LayoutDirection};
use crate::tree::{ContentType, Tree};

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        bail!("invalid usage");
    }

    let path = args.get(1).unwrap();
    let content_type = if path.ends_with(".yaml") {
        ContentType::Yaml
    } else if path.ends_with(".toml") {
        ContentType::Toml
    } else if path.ends_with(".json") {
        ContentType::Json
    } else {
        bail!("unsupported file type");
    };
    let path = PathBuf::from(path);

    let data = fs::read(path).context("read file")?;
    let data = String::from_utf8(data).context("parse utf8")?;

    let mut cfg = Config::default();
    cfg.parse().context("parse config")?;

    let tree = Tree::parse(&cfg, &data, content_type)?;

    let mut app = App::new(&cfg, tree, LayoutDirection::Horizontal);
    app.show()?;

    Ok(())
}
