use clap::Parser;

use crate::tree::ContentType;

#[derive(Parser, Debug)]
#[command(author, about)]
#[command(disable_version_flag = true)]
pub struct CommandArgs {
    /// The file path to read data. If not provided, read from stdin.
    pub path: Option<String>,

    /// The config file path to use. If not provided, we will try to load config from
    /// `~/.config/otree.toml`. The default config will be used when both this option is not
    /// set and `~/.config/otree.toml` does not exist.
    #[clap(long)]
    pub config: Option<String>,

    /// The data content type. If not provided, we will try to determine it based on the file
    /// extension. If reading data from stdin, or the file extension is not standard, this
    /// option must be set.
    #[clap(short, long)]
    pub content_type: Option<ContentType>,

    /// Force to use vertical layout.
    #[clap(short = 'V', long)]
    pub vertical: bool,

    /// Force to use horizontal layout.
    #[clap(short = 'H', long)]
    pub horizontal: bool,

    /// Print the version and exit.
    #[clap(short, long)]
    pub version: bool,
}
