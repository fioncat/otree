mod clipboard;
mod cmd;
mod config;
mod edit;
mod parse;
mod tree;
mod ui;

use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::process;

use anyhow::{bail, Context, Result};

use crate::cmd::CommandArgs;
use crate::config::Config;
use crate::parse::ContentType;
use crate::tree::Tree;
use crate::ui::{App, HeaderContext};

// Forbid large data size to ensure TUI performance
const MAX_DATA_SIZE: usize = 30 * 1024 * 1024;

fn run() -> Result<()> {
    let args = match CommandArgs::parse()? {
        Some(args) => args,
        None => return Ok(()),
    };

    let mut cfg = if args.ignore_config {
        Config::default()
    } else {
        Config::load(args.config.clone())?
    };

    args.update_config(&mut cfg);
    cfg.parse().context("parse config")?;

    if args.show_config {
        return cfg.show();
    }

    // The user can specify the content type manually, or we can determine it based on the
    // file extension. Another approach is to use file content (for example, if the file
    // content starts with '{', we can assume it is JSON). But this approach is not reliable
    // since the YAML is the superset of JSON, and the TOML is not easy to determine.
    let content_type = match args.content_type {
        Some(content_type) => content_type,
        None => {
            if args.path.is_none() {
                bail!("you must specify content type when reading data from stdin");
            }
            let path = PathBuf::from(args.path.as_ref().unwrap());

            let ext = path.extension();
            if ext.is_none() {
                bail!("cannot determine content type, missing extension in file path, you can specify it manually");
            }
            let ext = ext.unwrap().to_str();
            if ext.is_none() {
                bail!("invalid extension in file path");
            }

            match ext.unwrap() {
                "json" => ContentType::Json,
                "yaml" | "yml" => ContentType::Yaml,
                "toml" => ContentType::Toml,
                _ => bail!("unsupported file type, please specify content type manually"),
            }
        }
    };

    let data = match args.path.as_ref() {
        Some(path) => {
            let path = PathBuf::from(path);
            fs::read(path).context("read file")?
        }
        None => {
            if cfg!(target_os = "macos") {
                // Read from stdin is not supported on macOS.
                // See: <https://github.com/crossterm-rs/crossterm/issues/500>
                bail!(
                    "reading data from stdin is not supported on macOS, please read it from file"
                );
            }
            let mut data = Vec::new();
            io::stdin().read_to_end(&mut data).context("read stdin")?;
            data
        }
    };

    if data.len() > MAX_DATA_SIZE {
        bail!("the data size is too large, we limit the maximum size to 30 MiB to ensure TUI performance, you should try to reduce the read size");
    }

    // To make sure the data is utf8 encoded.
    let data = String::from_utf8(data).context("parse file utf8")?;

    let tree = Tree::parse(&cfg, &data, content_type).context("parse data")?;

    let mut app = App::new(&cfg, tree);

    if !cfg.header.disable {
        let header_ctx = HeaderContext::new(args.path, content_type, data.len());
        app.set_header(header_ctx);
    }

    ui::start(app)
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: {err:#}");
            process::exit(1);
        }
    }
}
