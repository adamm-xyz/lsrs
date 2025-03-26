#![warn(clippy::pedantic, clippy::nursery)]

use std::io::{self, Write};

use lsrs::cli::Flags;
use lsrs::entry::get_entries;

fn main() {
    // Get flags and entries from given path on command line
    let flags = Flags::from_args();
    match get_entries(flags.path.as_deref(), &flags) {
        Ok(entries) => {
            let mut stdout = io::stdout();
            if let Err(error) = entries.iter().enumerate().try_for_each(|(index, entry)| {
                // Comma separate
                if index != 0 && flags.stream_output {
                    write!(stdout, ", ")?;
                }
                // Print entries
                let result = entry.print_to(&mut stdout, &flags);
                if flags.stream_output {
                    stdout.flush()?;
                } else {
                    println!();
                }
                result
            }) {
                eprintln!("Error printing entries: {error}");
            }
        }
        Err(error) => eprintln!("Error listing entries: {error}"),
    }
}
