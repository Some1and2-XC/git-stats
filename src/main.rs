mod repo;
mod objects;

use std::{
    str::FromStr,
    env::args_os,
    ffi::OsString,
    path::PathBuf,
};

use anyhow::Result;

use crate::repo::Repo;
use crate::objects::GitObject;
use crate::objects::CommitObject;

const GIT_FOLDERNAME: &'static str = ".git";

fn full_commit_from_index(repository: &Repo, index: &str) -> CommitObject {

    let git_object = GitObject::from_index(repository, index).unwrap();
    let git_data_string = String::from_utf8_lossy(
        git_object
            .get_data()
            .unwrap()
            .as_slice()
        ).replace("\0", "\n");

    return CommitObject::from_str(&git_data_string);
}

fn main() -> Result<()> {


    // Gets the path from input args
    let os_string = args_os()
        .nth(1)
        .unwrap_or(OsString::from_str(".").unwrap());
    let path = PathBuf::from(&os_string);

    // Gets the repository path from the files
    let repository = Repo::from_path(&path).unwrap().enumerate_branches().unwrap();

    let res = repository.get_commit_from_index("ab").unwrap();
    println!("{:?}", res);

    return Ok(());
    // let branch = repository.get_branch("main".into()).unwrap();
    let branches = repository.branches.as_ref().unwrap().to_owned();
    let branch = repository.get_branch(branches[0].to_string_lossy().to_string()).unwrap();
    println!("{:?}", branch);

    let head = GitObject::from_index(&repository, &branch).unwrap();
    let head_data = String::from_utf8_lossy(
        head
            .get_data()
            .unwrap()
            .as_slice()
            ).replace("\0", "\n");
    let parsed_value = CommitObject::from_str(&head_data);
    let mut search_value = parsed_value.to_owned();

    for _ in 0..99999 {
        println!("{:?}", search_value);
        search_value = match search_value.parent {
            Some(v) => full_commit_from_index(&repository, &v),
            None => break,
        }
    }

    return Ok(());
}
