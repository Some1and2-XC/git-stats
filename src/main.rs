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

const GIT_FOLDERNAME: &'static str = ".git";

fn main() -> Result<()> {

    // Gets the path from input args
    let os_string = args_os()
        .nth(1)
        .unwrap_or(OsString::from_str(".").unwrap());
    let path = PathBuf::from(&os_string);

    // Gets the repository path from the files
    // And enumerates its branches
    let repository = Repo::from_pathbuf(&path)?
        .enumerate_branches()?
        ;

    let branch = repository.get_branch("main")?;
    let parent_index = GitObject::from_index(&repository, &branch.parent)?;

    let _ = repository
        .get_all_objects()?
        .iter()
        .map(|v| match v.initialize_from_data() {
                Ok(obj) => {
                    let _ = match obj {
                        objects::GitObjectType::Commit(commit) => {
                            // println!("Commit: {:?}", commit);
                        },
                    };
                    return 1;},
                Err(_) => 0,
        })
        .collect::<Vec<_>>()
        ;


    return Ok(());
}
