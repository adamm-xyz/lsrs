use clap::{ArgAction, Parser};

use std::path::PathBuf;


#[allow(
    clippy::struct_excessive_bools,
    reason = "this is not a state machine, but a set of flags"
)]
#[derive(Parser, Debug, Default)]
#[command(
    about = concat!(env!("CARGO_CRATE_NAME"), " - list directory contents"), 
    disable_help_flag = true
)]
pub struct Flags {
    /// show this help message
    #[arg(long, action(ArgAction::Help))]
    pub help: (),

    /// do not ignore entries starting with `.`
    #[arg(short = 'a', long = "all")]
    pub show_hidden: bool,

    /// show sizes of files; use -h for human-readable units
    #[arg(short = 's', long = "sizes")]
    pub show_size: bool,

    /// print sizes in human-readable units
    #[arg(short = 'h', long = "human")]
    pub human: bool,

    /// reverse order when sorting (-S, -t)
    #[arg(short = 'r', long = "reverse")]
    pub reverse_sort: bool,

    /// sort by file size, largest first (specify -r for smallest first)
    #[arg(short = 'S', long = "sort-size")]
    pub sort_by_size: bool,

    /// long listing (-l)
    #[arg(short = 'l', long)]
    pub long_listing: bool,

    /// sort by time modified, newest first (specify -r for oldest first)
    #[arg(short = 't', long = "sort-mtime")]
    pub sort_by_modified_time: bool,

    /// list files separated by `, `
    #[arg(short = 'm', long)]
    pub stream_output: bool,

    /// path to list entries from
    #[arg()]
    pub path: Option<PathBuf>,
}

impl Flags {
    /// Parse from `std::env::args_os()`, [exit][Error::exit] on error.
    // Wraps `clap::Parser` logic without direct trait imports
    // Equivalent to `Flags::parse()` here
    pub fn from_args() -> Self {
        Self::parse()
    }
}
