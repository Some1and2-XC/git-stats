#![allow(unused_imports)]

use std::{
    borrow::{
        BorrowMut,
        Cow
    }, collections::HashMap, env::args_os, ffi::OsString, io, net::TcpListener, path::PathBuf, str::FromStr
};

use anyhow::{anyhow, Result};
use clap::Parser;
use serde::{Serialize, Deserialize};
use chrono::prelude::{DateTime, Utc};

use git_stats::{
    objects::{
        blob::BlobObject,
        commit::CommitObject,
        tree::TreeObject,
        GitObjectType,
        GitObject,
        GitObjectAttributes,
    },
    Repo,
    macros::ok_or_continue,
};

mod cli;
mod server;

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

#[derive(Serialize, Deserialize)]
struct OutputValue {
    pub title: String,
    pub delta_t: u32,
    pub start: String,
    pub end: String,
}

/// Returns response data from CLI args
fn get_data(args: &cli::cli::CliArgs) -> Result<Vec<Vec<OutputValue>>> {
    // Gets the path from input args
    let os_string = OsString::from_str(&args.directory)?;
    let path = PathBuf::from(&os_string);

    // Gets the repository path from the files
    // And enumerates its branches
    let repo = Repo::from_pathbuf(&path)?;

    let mut branch = repo.get_branch(&args.branch).unwrap();

    let mut output_values: Vec<([i32;3], CommitObject)> = vec![];

    while let Some(parent_oid) = &branch.parent {
        let parent_branch = CommitObject::from_oid(&repo, parent_oid).unwrap();

        if let Some(email) = &args.email {
            match &branch.committer.email {
                Some(v) => {
                    if v != email {
                        branch = parent_branch;
                        continue;
                    }
                    ()
                },
                None => {
                    branch = parent_branch;
                    continue;
                },
            }
        }

        if let Some(committer) = &args.committer {
            if &branch.committer.name != committer {
                branch = parent_branch;
                continue;
            }
        }


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

    let output: Vec<Vec<OutputValue>> = windowed_values
        .iter()
        .map(|v| {
            return v.iter().map(|entry| {
                return OutputValue {
                    title: entry.1.message.trim().to_string(),
                    delta_t: entry.0[2] as u32,
                    end: DateTime::from_timestamp(entry.1.committer.timestamp as i64, 0).unwrap().to_rfc3339(),
                    start: DateTime::from_timestamp(entry.1.committer.timestamp as i64 - entry.0[2] as i64, 0).unwrap().to_rfc3339(),
                };
            })
            .collect::<Vec<OutputValue>>();
        }).collect();
    return Ok(output);
}

fn main() -> Result<()> {

    let args = cli::cli::CliArgs::parse();

    if args.server {
        let interface = format!("0.0.0.0:{}", &args.server_port);
        let server_directory = args.server_directory.clone();
        println!("Starting Server... on '{interface}' in directory: '{server_directory}'");
        let listener = TcpListener::bind(interface).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            server::handle_connection(stream, &server_directory.trim_end_matches("/"), &args);
        }
    } else {

        let output = get_data(&args)?;

        match &args.outfile {
            Some(v) => {
                let file = std::fs::File::create(v).unwrap();
                let writer = std::io::BufWriter::new(file);
                serde_json::to_writer(writer, &output).unwrap();
            },
            None => {
                serde_json::to_string(&output).unwrap();
            },
        }
    }


    return Ok(());
}
