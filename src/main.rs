mod raw_parsing;

use raw_parsing::{
    get_objects, get_repository, ls
};

use std::{
    env::args_os, ffi::OsString, path::{Path, PathBuf}, process::exit, str::FromStr
};

const GIT_FOLDERNAME: &'static str = ".git";

fn main() {


    // Gets the path from input args
    let os_string = args_os()
        .nth(1)
        .unwrap_or(OsString::from_str(".").unwrap());
    let path = Path::new::<OsString>(&os_string);

    // Gets all the files in that path
    let path_files = ls(path).unwrap();

    // Gets the repository path from the files
    let repository = match get_repository(path_files) {
        Some(v) => v,
        None => {
            println!("Not a git repository!");
            exit(1);
        },
    };

    // Get the git objects
    let objects = get_objects(repository);
    for object in objects {
        println!("{:?}", object);
    }
}
