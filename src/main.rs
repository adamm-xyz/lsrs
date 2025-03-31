#![warn(clippy::pedantic, clippy::nursery)]

use lsrs::cli::Flags;
use lsrs::entry::print_entries;

fn main() {
    // Get flags and entries from given path on command line
    let flags = Flags::from_args();
    let _ = print_entries(flags);
}
