mod raw_parsing;

use raw_parsing::{
    get_branch, get_git_objects, get_repository, ls, GitObject
};

use std::{
    env::args_os, ffi::OsString, path::{Path, PathBuf}, process::exit, str::FromStr
};

use crate::raw_parsing::CommitObject;

const GIT_FOLDERNAME: &'static str = ".git";

fn full_commit_from_index(repository: &PathBuf, index: &str) -> CommitObject {

    let git_object = GitObject::from_index(repository, index).unwrap();
    let git_data_string = git_object.get_data().replace("\0", "\n");

    return CommitObject::from_str(&git_data_string);
}

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

    let branch = get_branch(&repository, "main".into()).unwrap();
    println!("{:?}", branch);

    let head = GitObject::from_index(&repository, &branch).unwrap();
    let head_data = head.get_data().replace("\0", "\n");
    let parsed_value = CommitObject::from_str(&head_data);
    let mut search_value = parsed_value.to_owned();

    for _ in 0..9999 {
        println!("{:?}", search_value);
        search_value = match search_value.parent {
            Some(v) => full_commit_from_index(&repository, &v),
            None => break,
        }
    }

    return;
    println!("{:?}", parsed_value);
    println!("{}", head_data);

    // Get the git objects
    let objects = get_git_objects(repository);
    for object in objects {
        //. println!("{}", object.get_data());
    }
}
