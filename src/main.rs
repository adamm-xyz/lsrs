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
            return write!(writer, "{}", self.name.to_string_lossy().bold().red());
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
                format!("\t{} bytes", metadata.len()).color(color)
            )?;
        }
        Ok(())
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
    help: bool,
}

impl Flags {
    fn from_args(args: &[String]) -> Self {
        let has = |flag: &[&str]| flag.iter().any(|flag| args.iter().any(|arg| arg == flag));
        Self {
            show_hidden: has(&["-a", "--all"]),
            show_size: has(&["-s", "--sizes"]),
            help: has(&["-h", "--help"]),
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let (command, all_args) = args.split_first().unzip();
    let command = command.map_or(env!("CARGO_CRATE_NAME"), |command| command);
    let (mut last, args) = all_args.unwrap_or_default().split_last().unzip();
    let flags = if last.is_some_and(|last| last.starts_with('-')) {
        last = None;
        all_args.map(Flags::from_args).unwrap_or_default()
    } else {
        args.map(Flags::from_args).unwrap_or_default()
    };
    if flags.help {
        println!(
            "{command} - list directory contents
Usage: {command} [options] [PATH]
Options:
    -a, --all    \tdo not ignore entries starting with `.`\t[default: false]
    -s, --sizes    \tshow sizes of files in bytes\t\t[default: false]
    -h, --help    \tprint this help message",
        );
        return;
    }
    match get_entries(last, &flags) {
        Ok(entries) => {
            let mut stdout = io::stdout();
            for entry in entries {
                entry.print_to(&mut stdout, &flags).unwrap();
                println!();
            }
        }
        Err(e) => eprintln!("Error listing files: {e}"),
    }
}
