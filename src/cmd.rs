use anyhow::{bail, Result};
use clap::error::ErrorKind as ArgsErrorKind;
use clap::Parser;

use crate::config::{Config, LayoutDirection};
use crate::parse::ContentType;

#[derive(Parser, Debug)]
#[command(disable_version_flag = true)]
pub struct CommandArgs {
    /// The file to read data. On non-macOS systems, this can be omitted, and data
    /// will be read from stdin.
    pub path: Option<String>,

    /// The config file to use. Default will try to read `~/.config/otree.toml`.
    #[clap(long)]
    pub config: Option<String>,

    /// The data content type. If the file extension is one of
    /// ["json", "yaml", "yml", "toml"], this can be automatically inferred. In other
    /// cases, this is required.
    #[clap(short = 't', long)]
    pub content_type: Option<ContentType>,

    /// Don't show the header.
    #[clap(long)]
    pub disable_header: bool,

    /// Don't show the footer.
    #[clap(long)]
    pub disable_footer: bool,

    /// The header format.
    #[clap(short = 'f', long)]
    pub header_format: Option<String>,

    /// Use vertical layout.
    #[clap(short = 'V', long)]
    pub vertical: bool,

    /// Use horizontal layout.
    #[clap(short = 'H', long)]
    pub horizontal: bool,

    /// The tree widget size (percent), should be in range [10, 80].
    #[clap(short, long)]
    pub size: Option<u16>,

    /// Disable syntax highlighting in data block.
    #[clap(long)]
    pub disable_highlight: bool,

    /// Print loaded config (in toml).
    #[clap(long)]
    pub show_config: bool,

    /// Force to use the default config.
    #[clap(long)]
    pub ignore_config: bool,

    /// Print build info.
    #[clap(long)]
    pub build_info: bool,

    /// Print the version.
    #[clap(short, long)]
    pub version: bool,
}

impl CommandArgs {
    pub fn parse() -> Result<Option<Self>> {
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
                    return Ok(None);
                }

                eprintln!();
                bail!("parse command line args failed");
            }
        };

        if args.vertical && args.horizontal {
            bail!("invalid command line args, the vertical and horizontal cannot be used together");
        }

        if args.build_info {
            Self::show_build_info();
            return Ok(None);
        }

        if args.version {
            Self::show_version();
            return Ok(None);
        }

        Ok(Some(args))
    }

    pub fn update_config(&self, cfg: &mut Config) {
        if self.vertical {
            cfg.layout.direction = LayoutDirection::Vertical;
        }
        if self.horizontal {
            cfg.layout.direction = LayoutDirection::Horizontal;
        }

        if self.disable_header {
            cfg.header.disable = true;
        }

        if self.disable_footer {
            cfg.footer.disable = true;
        }

        if self.disable_highlight {
            cfg.data.disable_highlight = true;
        }

        if let Some(format) = self.header_format.as_ref() {
            cfg.header.format.clone_from(format);
        }

        if let Some(size) = self.size {
            cfg.layout.tree_size = size;
        }
    }

    fn show_version() {
        println!("otree {}", env!("OTREE_VERSION"));
    }

    fn show_build_info() {
        Self::show_version();

        println!();
        println!("commit: {}", env!("OTREE_SHA"));

        println!();
        println!("build type: {}", env!("OTREE_BUILD_TYPE"));
        println!("build target: {}", env!("OTREE_TARGET"));
        println!("build time: {}", env!("VERGEN_BUILD_TIMESTAMP"));

        println!();
        println!("rust version: {}", env!("VERGEN_RUSTC_SEMVER"));
        println!("rust channel: {}", env!("VERGEN_RUSTC_CHANNEL"));
        println!("llvm version: {}", env!("VERGEN_RUSTC_LLVM_VERSION"));

        if env!("VERGEN_CARGO_DEBUG") == "true" {
            println!();
            println!("[WARNING: Running in debug mode, the speed may be slow]");
        }
    }
}
