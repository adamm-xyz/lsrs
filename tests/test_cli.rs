use pretty_assertions::assert_eq; 
use clap::CommandFactory;

use lsrs::cli::Flags;


/// Catch problems earlier in the development cycle
#[test]
fn verify_cli_in_dev_cycle() {
    Flags::command().debug_assert();
}

#[test]
fn test_override_default_h_flag() {
    // this method WILL NOT exit when --help or --version (or short versions) are used
    // it will return a clap::Error, where the kind is a 
    // ErrorKind::DisplayHelp or ErrorKind::DisplayVersion respectively
    let res = Flags::command()
        .try_get_matches_from(["lsrs", "-h"]);
    assert!(res.is_ok());
}

#[test]
fn test_help_msg() {
    // remove all blank lines
    let info = Flags::command()
        .render_help()
        .to_string()
        .lines()
        .filter(|l| !l.is_empty()) 
        .map(|l| l.to_string()) 
        .collect::<Vec<String>>();

    assert_eq!(info, vec![
        "lsrs - list directory contents",
        "Usage: lsrs [OPTIONS] [PATH]",
        "Arguments:",
        "  [PATH]  path to list entries from",
        "Options:",
        "      --help           show this help message",
        "  -a, --all            do not ignore entries starting with `.`",
        "  -s, --sizes          show sizes of files; use -h for human-readable units",
        "  -h, --human          print sizes in human-readable units",
        "  -r, --reverse        reverse order when sorting (-S, -t)",
        "  -S, --sort-size      sort by file size, largest first (specify -r for smallest first)",
        "  -T, --show-time      show time (-T)",
        "  -t, --sort-mtime     sort by time modified, newest first (specify -r for oldest first)",
        "  -m, --stream-output  list files separated by `, `",
    ]);

}

// #[test]
// fn test_args_parsing() {
//     todo!()
// }
