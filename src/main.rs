#![warn(clippy::pedantic, clippy::nursery)]
use clap::{ArgAction, Parser};
use colored::{Color, Colorize};
use mime_guess::from_path;
use mime_guess::mime::{APPLICATION, IMAGE, TEXT, VIDEO};
use std::ffi::OsString;
use std::fs::Metadata;
use std::fs::{self, metadata};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, io};

/// Enum to represent directories or files
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FileType {
    Dir,
    File,
}

impl From<fs::FileType> for FileType {
    fn from(file_type: fs::FileType) -> Self {
        if file_type.is_dir() {
            Self::Dir
        } else {
            Self::File
        }
    }
}

impl FileType {
    /// Returns `true` if the file type is [`Dir`].
    ///
    /// [`Dir`]: FileType::Dir
    #[must_use]
    const fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }
}

/// Represents a File or a Dir with all the metadata
struct Entry {
    name: OsString,
    r#type: FileType,
    metadata: Option<Metadata>,
}

impl Entry {
    /// Prints the file or directory into writer depending on the flags
    fn print_to(&self, writer: &mut impl Write, flags: &Flags) -> io::Result<()> {
        // stream_output flag returnst the files and directories as a comma separated list
        if flags.stream_output {
            write!(writer, "{}", self.name.to_string_lossy())?;
            return Ok(());
        }

        if self.r#type.is_dir() {
            if flags.show_size {
                // Skip sizes on directories
                write!(writer, "\t")?;
            }
            return write!(writer, "{}/", self.name.to_string_lossy().bold().red());
        }

        let color = match from_path(&self.name).first_or_octet_stream().type_() {
            IMAGE => Color::Blue,
            TEXT => Color::Yellow,
            APPLICATION => Color::Green,
            VIDEO => Color::Cyan,
            _ => Color::Magenta,
        };

        if flags.show_size {
            if let Some(metadata) = &self.metadata {
                write!(
                    writer,
                    "{}",
                    if flags.human {
                        format!("{}\t", bytes_to_human(metadata.len()))
                    } else {
                        format!("{}\t", metadata.len())
                    }
                    .color(color)
                )?;
            }
        }

        write!(writer, "{}", self.name.to_string_lossy().color(color))?;

        Ok(())
    }
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
        return format!("{}{}", value, UNITS[index]);
    }
    format!("{:.1}{}", value, UNITS[index])
}

fn get_entries(dir_path: Option<&Path>, flags: &Flags) -> io::Result<Vec<Entry>> {
    // Convert `dir_path` to Path object
    let path = dir_path.as_ref().map(Path::new);

    // Check if it's a directory
    if let Some(path) = path {
        if !path.exists() {
            return Ok(Vec::new());
        }
        if !path.is_dir() {
            return Ok(vec![Entry {
                name: path.file_name().unwrap_or_default().to_os_string(),
                r#type: FileType::File,
                metadata: metadata(path).ok(),
            }]);
        }
    }

    let mut entries: Vec<_> = fs::read_dir(path.unwrap_or_else(|| Path::new(".")))?
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            if !flags.show_hidden && is_hidden_folder(&entry.path()) {
                return None;
            }
            entry.file_type().ok().map(|r#type| Entry {
                name,
                r#type: r#type.into(),
                metadata: entry.metadata().ok(),
            })
        })
        .collect();
    if flags.sort_by_size || flags.sort_by_modified_time {
        entries.sort_unstable_by(|a, b| {
            let key = |entry: &Entry| {
                let metadata = entry.metadata.as_ref();
                (
                    metadata
                        .map(std::fs::Metadata::len)
                        .map(|size| u64::MAX - size)
                        .filter(|_| flags.sort_by_size),
                    metadata
                        .map(std::fs::Metadata::modified)
                        .and_then(Result::ok)
                        .and_then(|time| time.elapsed().ok())
                        .filter(|_| flags.sort_by_modified_time),
                )
            };
            let mut ordering = Ord::cmp(&key(a), &key(b));
            if flags.reverse_sort {
                ordering = ordering.reverse();
            }
            Ord::cmp(&a.r#type, &b.r#type).then(ordering)
        });
    }
    Ok(entries)
}

fn is_hidden_folder(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes()[0] == b'.')
}

#[derive(Debug, Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "this is not a state machine, but a set of flags"
)]
#[derive(Parser)]
#[command(
    about = concat!(env!("CARGO_CRATE_NAME"), " - list directory contents"),
    disable_help_flag=true
)]
struct Flags {
    #[arg(long, action(ArgAction::Help), help = "show this help message")]
    help: (),
    #[arg(
        short = 'a',
        long = "all",
        help = "do not ignore entries starting with `.`"
    )]
    show_hidden: bool,
    #[arg(
        short = 's',
        long = "sizes",
        help = "show sizes of files; use -h for human-readable units"
    )]
    show_size: bool,
    #[arg(
        short = 'h',
        long = "human",
        help = "print sizes in human-readable units"
    )]
    human: bool,
    #[arg(
        short = 'r',
        long = "reverse",
        help = "reverse order when sorting (-S, -t)"
    )]
    reverse_sort: bool,
    #[arg(
        short = 'S',
        long = "sort-size",
        help = "sort by file size, largest first (specify -r for smallest first)"
    )]
    sort_by_size: bool,
    #[arg(
        short = 't',
        long = "sort-mtime",
        help = "sort by time modified, newest first (specify -r for oldest first)"
    )]
    sort_by_modified_time: bool,
    #[arg(short = 'm', long, help = "list files separated by `, `")]
    stream_output: bool,
    #[arg(help = "path to list entries from")]
    path: Option<PathBuf>,
}

fn main() {
    let flags = Flags::parse();
    match get_entries(flags.path.as_deref(), &flags) {
        Ok(entries) => {
            let mut stdout = io::stdout();
            if let Err(error) = entries.iter().enumerate().try_for_each(|(index, entry)| {
                if index != 0 && flags.stream_output {
                    write!(stdout, ", ")?;
                }
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
