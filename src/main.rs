use std::fs;
use std::path::Path;
use std::io;
use std::env;
use std::ffi::OsString;
use std::fs::FileType;
use colored::Colorize;

fn get_files(dir_path: &str) -> io::Result<Vec<(OsString,FileType)>> {
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

        if let Ok(file_type) = entry.file_type() {
            file_list.push((file_name,file_type));
        }
    }
    Ok(file_list)
}

fn strify_files(files: &Vec<(OsString,FileType)>) -> Vec<String> {
    let mut file_strs = Vec::new();
    for file in files {
        let (file_name,file_type) = file;
        if let Some(file_str) = file_name.to_str() {
            file_strs.push(file_str.to_string())
        }
    }
    file_strs
}


fn main() {
    let args: Vec<String> = env::args().collect();
    match get_files(&args[1]) {
        Ok(files) => {
            let file_strs = strify_files(&files);
            for file in file_strs{
                println!("{}", file);
            }
        }
        Err(e) => eprintln!("Error listing files: {}",e),
    }
}
