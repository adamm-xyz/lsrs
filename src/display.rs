use crate::entry::Entry;
use crate::cli::Flags;

use std::io::{self, Write};
use mime_guess::from_path;
use mime_guess::mime::{APPLICATION, IMAGE, TEXT, VIDEO};
use colored::{Color, Colorize};

pub fn print_entries(entries: Vec<Entry>, flags: Flags) -> io::Result<()> {
    let max_file_len = entries.iter()
        .map(|entry| if flags.human {
            bytes_to_human(entry.get_size()).chars().count()
        } else {
            entry.get_size().to_string().chars().count()
        })
        .max()
        .unwrap_or(6);
    let max_sym_len = entries.iter()
        .map(|entry| entry.get_links().chars().count())
        .max()
        .unwrap_or(2);

    let mut stdout = io::stdout();
    if let Err(error) = entries.iter().enumerate().try_for_each(|(index, entry)| {
        // Comma separate
        if index != 0 && flags.stream_output {
            write!(stdout, ", ")?;
        }
        // Print entries
        let result = entry.print_entry( &mut stdout, &flags, max_file_len, max_sym_len as usize);
        if flags.stream_output {
            stdout.flush()?;
        } else {
            println!();
        }
        result
    }) {
        eprintln!("Error printing entries: {error}");
        return Err(error);
    }
    Ok(())
}

impl Entry {
    /// Prints the file or directory into writer depending on the flags
    pub fn print_entry(&self, writer: &mut impl Write, flags: &Flags, max_file_len: usize, max_sym_len: usize) -> io::Result<()> {
        // stream_output flag returns the files and directories as a comma separated list
        if flags.stream_output {
            write!(writer, "{}", self.get_name())?;
            return Ok(());
        }

        if flags.long_listing {
            write!(writer, "{} ", self.get_permissions())?;
            write!(writer, "{} ", pad_str(self.get_links(),max_sym_len))?;
            write!(writer, "{} ", self.get_owners())?;
            write!(writer, "{} ",
                if flags.human {
                    format!("{}", pad_str(bytes_to_human(self.get_size()), max_file_len))
                } else {
                    format!("{} ", pad_str(self.get_size().to_string(),max_file_len))
                })?;
            write!(writer,"{} ", self.get_modified_time())?;
        }

        if self.is_folder() {
            if flags.show_size {
                // Skip sizes on directories
                write!(writer, "")?;
            }
            return write!(writer, "{}/", self.get_name().bold().red());
        }


        if flags.show_size && !flags.long_listing {
            write!(writer,"{}",
                if flags.human {
                    format!("{}\t", bytes_to_human(self.get_size()))
                } else {
                    format!("{}\t", self.get_size())
                })?;
        }

        // Entries are color coded based on file type
        let color = match from_path(&self.get_name()).first_or_octet_stream().type_() {
            IMAGE => Color::Blue,
            TEXT => Color::Yellow,
            APPLICATION => Color::Green,
            VIDEO => Color::Cyan,
            _ => Color::Magenta,
        };

        write!(writer, "{}", self.get_name().color(color))?;

        Ok(())
    }
}

fn pad_str(src: String, width: usize) -> String {
    format!("{:width$}", src, width = width)
}

/// Converts bytes into human readable format like 2.5KB
fn bytes_to_human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
    if bytes == 0 {
        return String::from("0B");
    }
    let index = (bytes.ilog(1024) as usize).min(UNITS.len() - 1);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "index is never more than `UNITS.len() - 1`"
    )]
    #[allow(
        clippy::cast_precision_loss,
        reason = "files probably won't be that big, and precision won't matter by that point"
    )]
    let value = bytes as f64 / 1024_f64.powi(index as i32);
    if index == 0 {
        format!("{}{}", value, UNITS[index])
    } else {
        format!("{:.1}{}", value, UNITS[index])
    }
}

