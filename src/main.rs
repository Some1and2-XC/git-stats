#![allow(unused_imports)]

use std::{
    borrow::{
        BorrowMut,
        Cow
    }, collections::HashMap, env::args_os, ffi::OsString, fs, io, net::TcpListener, path::PathBuf, str::FromStr
};

use anyhow::{anyhow, Result};
use clap::Parser;
use log::{debug, info, Level, Metadata, Record};
use serde::{Serialize, Deserialize};
use chrono::prelude::{DateTime, Utc};

use git_stats::{
    macros::ok_or_continue, objects::{
        blob::BlobObject, commit::CommitObject, tree::TreeObject, GitObject, GitObjectAttributes, GitObjectType
    }, packfiles::{self, Pack}, Repo
};

mod cli;
mod server;

/// Subtracts the values of tree1 from tree2.
/// Includes all the hashes from both trees combined.
/// The first tree is the newest tree.
/// The second tree is the previous tree.
/// The first return is the amount of lines removed.
/// The second return is the amount of lines added.
fn tree_diff(current_tree: HashMap<String, u32>, old_tree: HashMap<String, u32>) -> (i32, i32) {
    let mut all_keys = current_tree
        .keys()
        .collect::<Vec<&String>>();
    all_keys.extend(old_tree.keys().collect::<Vec<&String>>());

    let mut lines_added = 0;
    let mut lines_removed = 0;

    for key in all_keys {
        let new_value = current_tree.get(key).unwrap_or(&0).to_owned();
        let old_value = old_tree.get(key).unwrap_or(&0).to_owned();

        let delta = new_value as i32 - old_value as i32;

        if delta > 0 {
            lines_added += delta;
        } else {
            lines_removed -= delta;
        }
    }

    return (lines_removed, lines_added);
}

#[derive(Serialize, Deserialize, Debug)]
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
    let mut repo = Repo::from_pathbuf(&path)?;

    let mut branch = repo.get_branch(&args.branch)?;

    let mut output_values: Vec<([i32;3], CommitObject)> = vec![];

    while let Some(parent_oid) = &branch.parent {

        let parent_branch = match CommitObject::from_oid(&repo, parent_oid) {
            Ok(v) => v,
            Err(_) => {
                println!("Can't find branch: '{parent_oid}'");
                branch.parent = None;
                continue;
            },
        };

        if let Some(email) = &args.email {
            match &branch.committer.email {
                Some(v) => {
                    if v != email {
                        branch.parent = None;
                        continue;
                    }
                    ()
                },
                None => {
                    branch.parent = None;
                    continue;
                },
            }
        }

        if let Some(committer) = &args.committer {
            if &branch.committer.name != committer {
                branch.parent = None;
                continue;
            }
        }


        let difference = tree_diff(
            branch.get_tree(&repo)?.recurs_create_tree_line_count(&mut repo, ""),
            parent_branch.get_tree(&repo)?.recurs_create_tree_line_count(&mut repo, ""),
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

    std::env::set_var("RUST_BACKTRACE", "1");

    // Gets CLI arguments
    let args = cli::cli::CliArgs::parse();

    // Initializes the logger
    if let Some(level) = args.logs.to_level() {
        simple_logger::init_with_level(level).unwrap();
    }

    if args.server {
        let interface = format!("0.0.0.0:{}", &args.server_port);
        let server_directory = args.server_directory.clone();
        info!("Starting Server... on '{interface}' in directory: '{server_directory}'");
        let listener = TcpListener::bind(interface).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            server::handle_connection(stream, &server_directory.trim_end_matches("/"), &args);
        }
    } else {

        let files = std::fs::read_dir(
            PathBuf::from_str(&args.directory)
                .unwrap()
                .join(".git")
                .join("objects")
                .join("pack")
            ).unwrap();

        for file in files {
            // Only gets idx files
            let file_path = (&file.unwrap()).path();
            if !file_path.file_name().unwrap().to_string_lossy().to_string().ends_with("pack") {
                continue;
            }
            debug!("Found file: '{:?}'", file_path.file_name().unwrap());
            let mut packfile = packfiles::Pack::from_path(file_path.to_str().unwrap()).unwrap();
            packfile.run();
        }

        return Ok(());

        let output = get_data(&args)?;

        match &args.outfile {
            Some(v) => {
                let file = std::fs::File::create(v).unwrap();
                let writer = std::io::BufWriter::new(file);
                serde_json::to_writer(writer, &output).unwrap();
            },
            None => {
                println!("{}", serde_json::to_string(&output).unwrap());
            },
        }
    }


    return Ok(());
}
