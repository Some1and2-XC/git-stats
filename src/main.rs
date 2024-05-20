use std::{
    str::FromStr,
    env::args_os,
    ffi::OsString,
    path::PathBuf,
};

use anyhow::Result;
use git_stats::objects::{GitObject, GitObjectAttributes};
use git_stats::Repo;

use git_stats::objects::commit::CommitObject;

fn main() -> Result<()> {

    // Gets the path from input args
    let os_string = args_os()
        .nth(1)
        .unwrap_or(OsString::from_str(".")?);
    let path = PathBuf::from(&os_string);

    // Gets the repository path from the files
    // And enumerates its branches
    let repo = Repo::from_pathbuf(&path)?
        .enumerate_branches()?
        ;

    let mut branch = repo.get_branch("main")?;

    // let git_object = GitObject::from_oid(&repo, &branch.tree)?;
    // let tree = TreeObject::from_git_object(&git_object)?;
    // println!("{:?}", tree);

    return Ok(());
}
