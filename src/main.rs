#![allow(unused_imports)]

use std::{
    str::FromStr,
    env::args_os,
    ffi::OsString,
    path::PathBuf,
};

use anyhow::Result;
use git_stats::objects::blob::BlobObject;
use git_stats::objects::commit::CommitObject;
use git_stats::objects::GitObjectType;
use git_stats::objects::{GitObject, GitObjectAttributes};
use git_stats::Repo;
use git_stats::macros::ok_or_continue;
use git_stats::objects::tree::TreeObject;

fn recurse_get_values(repo: &Repo, git_object: &GitObject) -> Vec<GitObjectType> {
    let new_obj = match git_object.initialize_from_data().unwrap() {
        GitObjectType::Commit(v) => recurse_get_values(repo, &GitObject::from_oid(repo, &v.tree).unwrap()),
        GitObjectType::Blob(v) => vec![GitObjectType::Blob(v)],
        GitObjectType::Tree(v) => {
            println!("{:?}", v);
            let values = v.items.iter()
                .map(|item| {
                    return recurse_get_values(repo, &GitObject::from_oid(repo, &item.oid).unwrap());
                })
                .collect::<Vec<Vec<GitObjectType>>>();
            let mut new_array: Vec<GitObjectType> = vec![];
            for arr in values {
                for element in arr {
                    new_array.push(element);
                }
            }
            return new_array;
        },
    };

    return new_obj;
}

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

    let branch = repo.get_branch_oid("main")?;

    let values = recurse_get_values(&repo, &GitObject::from_oid(&repo, &branch).unwrap());
    println!("{}", values.len());
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
