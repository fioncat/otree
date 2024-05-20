mod cmd;
mod config;
mod interactive;
mod tree;

use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::process;

use anyhow::{bail, Context, Result};
use clap::error::ErrorKind as ArgsErrorKind;
use clap::Parser;

use crate::cmd::CommandArgs;
use crate::config::Config;
use crate::config::LayoutDirection;
use crate::interactive::app::App;
use crate::tree::{ContentType, Tree};

// Forbid large data size to ensure TUI performance
const MAX_DATA_SIZE: usize = 5 * 1024 * 1024;

fn run() -> Result<()> {
    let args = match CommandArgs::try_parse() {
        Ok(args) => args,
        Err(err) => {
            err.use_stderr();
            err.print().unwrap();
            if matches!(
                err.kind(),
                ArgsErrorKind::DisplayHelp
                    | ArgsErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                    | ArgsErrorKind::DisplayVersion
            ) {
                return Ok(());
            }

            eprintln!();
            bail!("parse command line args failed");
        }
    };

    let config_path = args.config.clone();
    let mut cfg = Config::load(config_path)?;

    if args.vertical && args.horizontal {
        bail!("invalid command line args, the vertical and horizontal cannot be used together");
    }
    if args.vertical {
        cfg.layout = LayoutDirection::Vertical;
    }
    if args.horizontal {
        cfg.layout = LayoutDirection::Horizontal;
    }

    // The user can specify the content type manually, or we can determine it based on the
    // file extension. Another approach is to use file content (for exmaple, if the file
    // content starts with '{', we can assume it is json). But this approach is not reliable
    // since the yaml is the superset of json, and the toml is not easy to determine.
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
            let mut data = Vec::new();
            io::stdin().read_to_end(&mut data).context("read stdin")?;
            data
        }
    };

    if data.len() > MAX_DATA_SIZE {
        // Now the items need to be cloned every time the tree is rendered, which is a
        // performance bottleneck. So we add this limit to ensure the performance of the TUI.
        // After <https://github.com/EdJoPaTo/tui-rs-tree-widget/issues/35> is resolved, the
        // limit here can be removed or increased.
        bail!("the data size is too large, we limit the maximum size to 5 MiB to ensure TUI performance, you should try to reduce the read size");
    }

    // To make sure the data is utf8 encoded.
    let data = String::from_utf8(data).context("parse file utf8")?;

    let tree = Tree::parse(&cfg, &data, content_type).context("parse file")?;

    let mut app = App::new(&cfg, tree);
    app.show().context("show tui")?;

    Ok(())
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
