#![allow(unused_imports)]

use std::borrow::{BorrowMut, Cow};
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

mod cli;

/// Subtracts the values of tree1 from tree2.
/// Includes all the hashes from both trees combined.
/// The first tree is the newest tree.
/// The second tree is the previous tree.
/// The first return is the amount of lines removed.
/// The second return is the amount of lines added.
fn tree_diff(current_tree: HashMap<String, BlobObject>, old_tree: HashMap<String, BlobObject>) -> (i32, i32) {
    let mut all_keys = current_tree
        .keys()
        .collect::<Vec<&String>>();
    all_keys.extend(old_tree.keys().collect::<Vec<&String>>());

    let mut lines_added = 0;
    let mut lines_removed = 0;

    for key in all_keys {
        let new_value = match current_tree.get(key) {
            Some(v) => v.line_amnt(),
            None => 0,
        };
        let old_value = match old_tree.get(key) {
            Some(v) => v.line_amnt(),
            None => 0,
        };

        let delta = new_value as i32 - old_value as i32;

        if delta > 0 {
            lines_added += delta;
        } else {
            lines_removed -= delta;
        }
    }

    return (lines_removed, lines_added);
}

fn main() -> Result<()> {

    // Gets the path from input args
    let os_string = args_os()
        .nth(1)
        .unwrap_or(OsString::from_str(".")?);
    let path = PathBuf::from(&os_string);

    // Gets the repository path from the files
    // And enumerates its branches
    let repo = Repo::from_pathbuf(&path)?;

    let mut branch = repo.get_branch("main").unwrap();

    let mut output_values: Vec<([i32;3], CommitObject)> = vec![];

    while let Some(parent_oid) = &branch.parent {
        let parent_branch = CommitObject::from_oid(&repo, parent_oid).unwrap();
        let difference = tree_diff(
            branch.get_tree(&repo).unwrap().recurs_create_tree(&repo, ""),
            parent_branch.get_tree(&repo).unwrap().recurs_create_tree(&repo, ""),
        );

        let time_difference = branch.committer.timestamp - parent_branch.committer.timestamp;

        output_values.push(([difference.0, difference.1, time_difference as i32], branch));

        branch = parent_branch;
    }

    let removed_average: f32 = output_values
        .iter()
        .map(|(v, _)| (v[0] as f32 / v[2] as f32))
        .sum::<f32>() / output_values.len() as f32
        ;

    let added_average: f32 = output_values
        .iter()
        .map(|(v, _)| (v[1] as f32 / v[2] as f32))
        .sum::<f32>() / output_values.len() as f32
        ;

    let windowed_values: Vec<Vec<([i32;3], CommitObject)>> = output_values
        .split_inclusive(|&(v, _)| v[2] > 3600 * 5)
        .collect::<Vec<&[([i32;3], CommitObject)]>>()
        .iter_mut()
        .map(|v| {
            let mut items = v.to_owned();
            let item = items.last_mut().unwrap();
            // Changes timestamp to projected amount
            item.0[2] = ((item.0[0] as f32 * removed_average + item.0[1] as f32 * added_average) / 2.0) as i32;
            return items;
        })
        .collect()
        ;

    fn get_time(mut n: f32) -> String{
        if n < 60.0 {
            return format!("{n:.0}S");
        }
        n /= 60.0;
        if n < 60.0 {
            return format!("{n:.2}M");
        }
        n /= 60.0;
        if n < 24.0 {
            return format!("{n:.2}H");
        }
        n /= 24.0;
        return format!("{n:.2}D");
    }

    let _: Vec<()> = windowed_values
        .iter()
        .map(|v| {
            for entry in v {
                println!("{} {} ({})", get_time(entry.0[2] as f32), entry.1.message.trim(), entry.1.committer.timestamp);
            }
            println!("\t[END OF GROUP]\n");

            return ();
        }).collect();

    return Ok(());
}
