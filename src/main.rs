#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::struct_excessive_bools)]
use colored::{Color, Colorize};
use mime_guess::from_path;
use mime_guess::mime::{APPLICATION, IMAGE, TEXT, VIDEO};
use std::ffi::{OsStr, OsString};
use std::fs::Metadata;
use std::fs::{self, metadata};
use std::io::Write;
use std::path::Path;
use std::{env, io};

#[derive(Debug, Clone, Copy)]
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

struct Entry {
    name: OsString,
    r#type: FileType,
    metadata: Option<Metadata>,
}

impl Entry {
    fn print_to(&self, writer: &mut impl Write, flags: &Flags) -> io::Result<()> {
        if self.r#type.is_dir() {
            return write!(writer, "{}/", self.name.to_string_lossy().bold().red());
        }
        let color = match from_path(&self.name).first_or_octet_stream().type_() {
            IMAGE => Color::Blue,
            TEXT => Color::Yellow,
            APPLICATION => Color::Green,
            VIDEO => Color::Cyan,
            _ => Color::Magenta,
        };
        write!(writer, "{}", self.name.to_string_lossy().color(color))?;
        if !flags.show_size {
            return Ok(());
        }
        if let Some(metadata) = &self.metadata {
            write!(
                writer,
                "{}",
                if flags.human_readable {
                    format!("\t{}", human_readable_size(metadata.len())).color(color)
                } else {
                    format!("\t{} bytes", metadata.len()).color(color)
                }
            )?;
        }
        Ok(())
    }
}

fn human_readable_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    #[allow(clippy::cast_precision_loss)]
    let size = bytes as f64;

    if size >= GB {
        format!("{:.2} GiB", size / GB)
    } else if size >= MB {
        format!("{:.2} MiB", size / MB)
    } else if size >= KB {
        format!("{:.2} KiB", size / KB)
    } else {
        format!("{bytes} bytes")
    }
}

fn get_entries(dir_path: Option<&impl AsRef<OsStr>>, flags: &Flags) -> io::Result<Vec<Entry>> {
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

    Ok(fs::read_dir(path.unwrap_or_else(|| Path::new(".")))?
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
        .collect())
}

fn is_hidden_folder(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes()[0] == b'.')
}

#[derive(Debug, Default)]
struct Flags {
    show_hidden: bool,
    show_size: bool,
    human_readable: bool,
    help: bool,
}

impl Flags {
    fn from_args(args: &[String]) -> (Self, Option<String>) {
        let mut flags = Self::default();
        let mut path = None;

        for arg in args {
            match arg.as_str() {
                "-a" | "--all" => flags.show_hidden = true,
                "-s" | "--sizes" => flags.show_size = true,
                "-H" | "--human-readable" => flags.human_readable = true,
                "-h" | "--help" => flags.help = true,
                _ if !arg.starts_with('-') => path = Some(arg.clone()),
                _ => {}
            }
        }
        (flags, path)
    }
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let (flags, path) = Flags::from_args(&args);

    if flags.help {
        let command = env!("CARGO_CRATE_NAME");
        println!(
            "{command} - list directory contents
Usage: {command} [options] [PATH]
Options:
    -a, --all\t\t\tdo not ignore entries starting with `.` [default: false]
    -s, --sizes\t\t\tshow sizes of files in bytes [default: false]
    -H, --human-readable\tshow sizes of files in human readable format [default: false]
    -h, --help\t\t\tprint this help message",
        );
        return;
    }
    match get_entries(path.as_ref(), &flags) {
        Ok(entries) => {
            let mut stdout = io::stdout();
            if let Err(error) = entries.iter().try_for_each(|entry| {
                let result = entry.print_to(&mut stdout, &flags);
                println!();
                result
            }) {
                eprintln!("Error printing entries: {error}");
            }
        }
        Err(error) => eprintln!("Error listing entries: {error}"),
    }
}
