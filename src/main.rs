#![warn(clippy::pedantic, clippy::nursery)]

use lsrs::cli::Flags;
use lsrs::display::print_entries;
use lsrs::entry::get_entries;

fn main() {
    // Get flags and entries from given path on command line
    let flags = Flags::from_args();
    match get_entries(flags.path.as_deref(), &flags) {
        Ok(entries) => {
            match print_entries(entries,flags) {
                Ok(()) => (),
                Err(error) => eprintln!("{error}")
            }
        },
        Err(error) => eprintln!("{error}")
    }
}
