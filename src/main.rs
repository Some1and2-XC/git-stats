#![allow(unused_imports)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::{
    str::FromStr,
    env::args_os,
    ffi::OsString,
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use git_stats::objects::blob::BlobObject;
use git_stats::objects::commit::CommitObject;
use git_stats::objects::GitObjectType;
use git_stats::objects::{GitObject, GitObjectAttributes};
use git_stats::Repo;
use git_stats::macros::ok_or_continue;
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

    let branch = repo.get_branch("main").unwrap();
    let _ = CommitObject::from_oid(&repo, &branch.parent.clone().unwrap());
    let tree = branch.get_tree(&repo).unwrap();
    let values = tree.recurs_create_tree(&repo, "");
    println!("{}", values.len());

    for (filename, value) in values.iter() {
        println!("{} {}", filename, value.line_amnt());
    }

    /*
    for value in values {
        match value {
            GitObjectType::Blob(v) => println!("{}", v),
            _ => println!("WARNING, NOT FOUND!"),
        }
    }
    */

    return Ok(());
}
