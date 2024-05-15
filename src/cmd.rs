use clap::Parser;

use crate::tree::ContentType;

#[derive(Parser, Debug)]
#[command(author, about)]
#[command(disable_version_flag = true)]
pub struct CommandArgs {
    pub path: String,

    #[clap(short, long)]
    pub config: Option<String>,

    #[clap(short = 't', long)]
    pub content_type: Option<ContentType>,

    #[clap(short = 'V', long)]
    pub vertical: bool,

    #[clap(short = 'H', long)]
    pub horizontal: bool,

    #[clap(short, long)]
    pub version: bool,
}
