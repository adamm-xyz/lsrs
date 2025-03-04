use std::fs;
use std::path::Path;
use std::io;
use std::env;
use std::ffi::OsString;
use std::fs::{FileType,Metadata};
use colored::{Colorize,ColoredString,Color};
use mime_guess::from_path;

struct FileInfo {
    name: String,
    size: u64,
    is_dir: bool,
    is_hidden: bool,
    color: Color,
}

fn get_file_info_list(dir_path: &str, flags: &Flags) -> io::Result<Vec<FileInfo>> {
    //convert to Path obj
    let path = Path::new(dir_path);

    //check if directory
    if !path.is_dir(){
        return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Provided path is not a directory"
        ));
    }

    //contents of directory
    let mut file_list = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if let Some(file_str) = file_name.to_str() {
            if let Ok(file_type) = entry.file_type() {
                if let Ok(file_data) = entry.metadata() {
                    let file_string = file_str.to_string();
                    let file_size = file_data.len();
                    let file_is_dir = file_type.is_dir();
                    let file_color = set_file_color(file_str);
                    file_list.push(
                        FileInfo {
                            name: file_string,
                            size: file_size,
                            is_dir: file_is_dir,
                            is_hidden: false,
                            color: file_color
                        }
                    )
                }
            }
        }

    }
    Ok(file_list)
}

fn get_files(dir_path: &str, flags: &Flags) -> io::Result<Vec<(OsString,FileType, Metadata)>> {
    //convert to Path obj
    let path = Path::new(dir_path);

    //check if directory
    if !path.is_dir(){
        return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Provided path is not a directory"
        ));
    }

    //contents of directory
    let mut file_list = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if flags.ignore_hidden {
            if !is_hidden_folder(entry.path().as_path()) {
                if let Ok(file_type) = entry.file_type() {
                    if let Ok(file_data) = entry.metadata() {
                        file_list.push((file_name,file_type,file_data));
                    }
                }
            }
        } else {
            if let Ok(file_type) = entry.file_type() {
                if let Ok(file_data) = entry.metadata() {
                    file_list.push((file_name,file_type,file_data));
                }
            }
        }

    }
    Ok(file_list)
}


fn is_hidden_folder(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

fn get_file_mime(filename: &str) -> String {
    let file_mime = from_path(filename).first_or_octet_stream();
    let file_mime_type = file_mime.type_();
    file_mime_type.to_string()
}

fn set_file_color(filename: &str) -> Color {
   match get_file_mime(filename).as_str() {
       "image" => {
           Color::Blue
       }
       "text" => {
           Color::Yellow
       }
       "application" => {
           Color::Green
       }
       "video" => {
           Color::Cyan
       }
       &_ => Color::Magenta
   }
}

fn strify_files(files: &Vec<(OsString,FileType, Metadata)>, flags: &Flags) -> Vec<ColoredString> {
    let mut file_strs = Vec::new();
    for file in files {
        let (file_name,file_type,file_data) = file;
        if let Some(file_str) = file_name.to_str() {
            let mut file_str_colored = file_str.to_string().red();

            if file_type.is_dir() {
                file_str_colored = file_str_colored.bold();
            } else {
                if flags.show_size {
                    let file_size = file_data.len().to_string();
                    //file_str_colored = set_file_color(file_str) + String::from(" ") + file_size;
                    file_str_colored = format!("{} - {}",file_str,file_size).into();
                    file_str_colored = file_str_colored.blue();
                    //println!("{:?}",get_file_mime(file_str));
                } else {
                    file_str_colored = file_str_colored.blue();
                }
            }
            file_strs.push(file_str_colored);
            //file_strs.push(file_str.to_string().red());
        }
    }
    file_strs
}

struct Flags {
    ignore_hidden: bool,
    show_size: bool,
}


fn parse_flags(user_input: &String) -> Flags {
    let mut program_flags = Flags {
        ignore_hidden: true,
        show_size: false,
    };

    if user_input.contains("a") {
        program_flags.ignore_hidden = false;
    }
    if user_input.contains("s") {
        program_flags.show_size = true;
    }

    program_flags
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut flags = parse_flags(&String::from(""));
    if args.len() >= 3 {
        flags = parse_flags(&args[2]);
    }
    match get_files(&args[1],&flags) {
        Ok(files) => {
            let file_strs = strify_files(&files,&flags);
            for file in file_strs{
                println!("{}", file);
            }
        }
        Err(e) => eprintln!("Error listing files: {}",e),
    }
}
