use std::{
    str::FromStr,
    env::args_os,
    ffi::OsString,
    path::PathBuf,
};

use anyhow::Result;
use git_stats::objects::commit::CommitObject;
use git_stats::objects::{GitObject, GitObjectAttributes};
use git_stats::Repo;
use git_stats::objects::tree::TreeObject;

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

    let branch = repo.get_branch("main")?;

    println!("{:?}", &branch.tree);
    let git_object = GitObject::from_oid(&repo, &branch.tree)?;
    let tree = *TreeObject::from_git_object(&git_object)?;
    println!("{:?}", tree);

    /*
    loop {

        let git_object = GitObject::from_oid(&repo, &branch.tree)?;
        // let tree = *TreeObject::from_git_object(&git_object)?;
        println!("{:?}", branch);

        if branch.parent == None { break; }
        branch = CommitObject::from_oid(&repo, &branch.parent.unwrap()).unwrap();
    }
    */

    return Ok(());
}
