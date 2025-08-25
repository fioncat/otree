mod clipboard;
mod cmd;
mod config;
mod debug;
mod edit;
mod live_reload;
mod parse;
mod tree;
mod ui;

use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;

use anyhow::{bail, Context, Result};
use live_reload::FileWatcher;

use crate::cmd::CommandArgs;
use crate::config::Config;
use crate::parse::ContentType;
use crate::tree::Tree;
use crate::ui::{App, HeaderContext};

fn run() -> Result<()> {
    let Some(args) = CommandArgs::parse()? else {
        return Ok(());
    };

    let mut cfg = if args.ignore_config {
        Config::default()
    } else {
        Config::load(args.config.clone())?
    };

    args.update_config(&mut cfg);
    cfg.parse().context("parse config")?;

    let cfg = Rc::new(cfg);

    if args.show_config {
        return cfg.show();
    }

    if let Some(ref debug_file) = args.debug {
        debug::set_file(debug_file.clone());
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
                "xml" => ContentType::Xml,
                "jsonl" => ContentType::Jsonl,
                _ => bail!("unsupported file type, please specify content type manually"),
            }
        }
    };

    let max_data_size = args.max_data_size.unwrap_or(cfg.data.max_data_size) * 1024 * 1024;
    let mut fw = None;
    let data = match args.path.as_ref() {
        Some(path) => {
            let path = PathBuf::from(path);
            if args.live_reload {
                fw = Some(FileWatcher::new(
                    path.clone(),
                    cfg.clone(),
                    content_type,
                    max_data_size,
                ));
            }
            fs::read(path).context("read file")?
        }
        None => {
            let mut data = Vec::new();
            io::stdin().read_to_end(&mut data).context("read stdin")?;
            data
        }
    };

    if data.len() > max_data_size {
        bail!("the data size is too large, we limit the maximum size to {} to ensure TUI performance, you should try to reduce the read size. HINT: You can use command line arg `--max-data-size` or config option `data.max_data_size` to modify this limitation", humansize::format_size(max_data_size, humansize::BINARY));
    }

    // To make sure the data is utf8 encoded.
    let data = String::from_utf8(data).context("parse file utf8")?;

    let tree = Tree::parse(cfg.clone(), &data, content_type).context("parse data")?;

    let mut app = App::new(cfg.clone(), tree, fw);

    if !cfg.header.disable {
        let header_ctx = HeaderContext::new(args.path, content_type, data.len());
        app.set_header(&header_ctx);
    }

    ui::start(app)
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:#}");
        process::exit(1);
    }
}
