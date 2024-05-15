mod cmd;
mod config;
mod interactive;
mod tree;

use std::fs;
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

    let path = PathBuf::from(&args.path);

    let content_type = match args.content_type {
        Some(content_type) => content_type,
        None => {
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

    let data = fs::read(path).context("read file")?;
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
