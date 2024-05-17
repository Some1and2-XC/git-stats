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

fn main() -> Result<()> {


    // Gets the path from input args
    let os_string = args_os()
        .nth(1)
        .unwrap_or(OsString::from_str(".").unwrap());
    let path = PathBuf::from(&os_string);

    // Gets the repository path from the files
    // And enumerates its branches
    let repository = Repo::from_path(&path)?
        .enumerate_branches()?
        ;

    let branch = repository.get_branch("main")?;
    println!("{:?}", branch);

    return Ok(());
}
