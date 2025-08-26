use anyhow::{bail, Result};
use clap::error::ErrorKind as ArgsErrorKind;
use clap::Parser;

use crate::config::{Config, LayoutDirection};
use crate::parse::ContentType;

#[expect(clippy::struct_excessive_bools)] // bearable, just commandline args constructed once
#[derive(Parser, Debug)]
#[command(disable_version_flag = true)]
pub struct CommandArgs {
    /// The file to read data. If this is empty, read from stdin.
    pub path: Option<String>,

    /// The config file to use. Default will try to read `~/.config/otree.toml`.
    #[clap(long)]
    pub config: Option<String>,

    #[expect(clippy::doc_link_with_quotes)] // false positive, it's just a list
    /// The data content type. If the file extension is one of
    /// ["json", "yaml", "yml", "xml", "toml", "hcl", "jsonl"], this can be automatically
    // inferred. In other cases, this is required.
    #[clap(short = 't', long)]
    pub content_type: Option<ContentType>,

    /// Convert data to another content type and print to stdout
    #[clap(short = 'o', long)]
    pub to: Option<ContentType>,

    /// Don't show the header.
    #[clap(long)]
    pub disable_header: bool,

    /// Don't show the footer.
    #[clap(long)]
    pub disable_footer: bool,

    /// Disable the filter.
    #[clap(long)]
    pub disable_filter: bool,

    /// Disable highlight the selected item in tree.
    #[clap(long)]
    pub tree_disable_selected_highlight: bool,

    /// The symbol to indicate the selected item in tree.
    #[clap(long)]
    pub tree_selected_symbol: Option<String>,

    /// Ignore case when filtering.
    #[clap(long)]
    pub filter_ignore_case: bool,

    /// Hide items that do not match the filter.
    #[clap(long)]
    pub filter_exclude_mode: bool,

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

    /// Print loaded config.
    #[clap(long)]
    pub show_config: bool,

    /// Force to use the default config.
    #[clap(long)]
    pub ignore_config: bool,

    /// Print build info.
    #[clap(long)]
    pub build_info: bool,

    /// Limit data size to read. In MiB
    #[clap(long)]
    pub max_data_size: Option<usize>,

    /// Watch file changes, reload tui if updates
    #[clap(long, short = 'R')]
    pub live_reload: bool,

    /// Write debug logs into this file. The file will be created if not exists.
    #[clap(long)]
    pub debug: Option<String>,

    /// Print version.
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

        if self.disable_filter {
            cfg.filter.disable = true;
        }

        if self.tree_disable_selected_highlight {
            cfg.tree.disable_selected_highlight = true;
        }

        if let Some(ref symbol) = self.tree_selected_symbol {
            cfg.tree.selected_symbol.clone_from(symbol);
        }

        if self.filter_ignore_case {
            cfg.filter.ignore_case = true;
        }

        if self.filter_exclude_mode {
            cfg.filter.exclude_mode = true;
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
