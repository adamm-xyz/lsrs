
use chrono::{Local, TimeZone};
use libc::{S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};
use std::ffi::OsString;
use std::fs::{self, metadata, Metadata};
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::time::SystemTime;
use users::{get_group_by_gid, get_user_by_uid};

use crate::cli::Flags;

/// Enum to represent directories or files
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileType {
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
    pub const fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }
}

/// Represents a File or a Dir with all the metadata
pub struct Entry {
    pub name: OsString,
    pub r#type: FileType,
    pub metadata: Metadata,
}

impl Entry {
    /// Gets permissions of entry
    pub fn get_permissions(&self) -> String {
        parse_permissions(self.metadata.permissions().mode())
    }
    /// Gets num of links
    pub fn get_links(&self) -> String {
        format!("{}", self.metadata.nlink())
    }

    /// Get owner and group of entry
    pub fn get_owners(&self) -> String {
        get_file_owner_and_group(&self.metadata)
    }

    /// Get modified (local) time of entry
    pub fn get_modified_time(&self) -> String {
        match self.metadata.modified() {
            Ok(mod_time) => get_file_date(mod_time),
            Err(e) => format!("Error: {}",e)
        }
    }
}

pub fn get_entries(dir_path: Option<&Path>, flags: &Flags) -> io::Result<Vec<Entry>> {
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
                metadata: if let Some(meta) = metadata(path).ok() {
                    meta
                } else {
                    eprintln!("ERROR: Could not retrieve metadata!");
                    std::process::exit(1)
                },
            }]);
        }
    }

    // Collect entries into vector, ignoring hidden entries if show_hidden is false
    let mut entries: Vec<_> = fs::read_dir(path.unwrap_or_else(|| Path::new(".")))?
        .filter_map(|entry_result| {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Warning: Could not access entry: {}", e);
                    return None;
                }
            };

            let name = entry.file_name();
            if !flags.show_hidden && is_hidden_folder(&entry.path()) {
                return None;
            }

            let file_type = match entry.file_type() {
                Ok(ft) => ft.into(),
                Err(e) => {
                    eprintln!("Warning: Could not get file type: {}", e);
                    return None;
                }
            };

            let metadata = match entry.metadata() {
                Ok(meta) => meta,
                Err(e) => {
                    eprintln!("Warning: Could not retrieve metadata: {}", e);
                    return None;
                }
            };

            Some(Entry {
                name,
                r#type: file_type,
                metadata,
            })
        })
        .collect();


    // Sort the entries by relevant flag (by size or by time modified)
    if flags.sort_by_size || flags.sort_by_modified_time {
        entries.sort_unstable_by(|a, b| {
            let key = |entry: &Entry| {
                let metadata = &entry.metadata;
                (
                    if flags.sort_by_size {
                        Some(u64::MAX - metadata.len())
                    } else {
                        None
                    },
                    if flags.sort_by_modified_time {
                        metadata.modified()
                            .ok()
                            .and_then(|time| time.elapsed().ok())
                    } else {
                        None
                    }
                )
            };
            let mut ordering = Ord::cmp(&key(a), &key(b));
            // Reversing
            if flags.reverse_sort {
                ordering = ordering.reverse();
            }
            Ord::cmp(&a.r#type, &b.r#type).then(ordering)
        });
    }
    Ok(entries)
}

// Checks if given Path is 'hidden' (starts with '.')
fn is_hidden_folder(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes()[0] == b'.')
}

/// Helper functions to get and parse permissions of entries
/// Credit to Matthias Endler at endler.dev
fn parse_permissions(mode: u32) -> String {
    let user = triplet(mode, S_IRUSR, S_IWUSR, S_IXUSR);
    let group = triplet(mode, S_IRGRP, S_IWGRP, S_IXGRP);
    let other = triplet(mode, S_IROTH, S_IWOTH, S_IXOTH);
    [user, group, other].join("")
}

fn triplet(mode: u32, read: u32, write: u32, execute: u32) -> String {
    match (mode & read, mode & write, mode & execute) {
        (0, 0, 0) => "---",
        (_, 0, 0) => "r--",
        (0, _, 0) => "-w-",
        (0, 0, _) => "--x",
        (_, 0, _) => "r-x",
        (_, _, 0) => "rw-",
        (0, _, _) => "-wx",
        (_, _, _) => "rwx",
    }
    .to_string()
}


/// Gets the owner and group names associated with a file
pub fn get_file_owner_and_group(meta: &Metadata) -> String {
    // Get owner and group IDs
    let uid = meta.uid();
    let gid = meta.gid();

    // Look up the user and group names
    let owner_name = get_user_by_uid(uid)
        .map(|user| user.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| uid.to_string());

    let group_name = get_group_by_gid(gid)
        .map(|group| group.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| gid.to_string());

    format!("{} {}", owner_name, group_name)
}

/// Converts SystemTime of file metadata to readable string EX: Sep 10 14:23
fn get_file_date(modified_time: SystemTime) -> String {
    match modified_time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(time_since_epoch) => {
            let secs = time_since_epoch.as_secs() as i64;
            let nsecs = time_since_epoch.subsec_nanos();
            let timestamp = Local.timestamp_opt(secs, nsecs).unwrap();
            timestamp.format("%b %d %H:%M").to_string()
        }
        Err(e) => format!("Error: {e:?}"),
    }
}

