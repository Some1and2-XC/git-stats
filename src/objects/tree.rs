use anyhow::{anyhow, ensure, Result};
use fancy_regex::Regex;
use super::{
    GitObjectAttributes,
    get_type_size_and_data,
};

use crate::macros::ok_or_continue;

// mode: 100644 for a regular file, 100755 executable; 040000: tree; 120000: symlink; 160000: gitlink

/// Object that represents a commit
/// Designed to be initialized using the [`CommitObject::from_str`] function.
#[derive(Debug, Clone)]
pub struct TreeObject {}

impl TreeObject {
    fn new() -> Self {
        return Self {};
    }
}

pub fn get_number_filename_and_data(in_str: &str) -> Result<(i32, String, Vec<u8>)> {

    let split_data = in_str
        .splitn(2, "\0")
        .map(|v| v)
        .collect::<Vec<&str>>();

    ensure!(split_data.len() == 2, anyhow!(
        "Null character not found! Data: {}",
        in_str,
    ));

    let git_data: Vec<u8> = split_data[1].bytes().collect();

    // Gets meta segment
    let meta = split_data[0]
        .splitn(2, " ")
        .map(|v| v)
        .collect::<Vec<&str>>()
        ;

    ensure!(meta.len() == 2, anyhow!(
        "Git Data type isn't of length 2! Data: '{:?}'",
        meta,
    ));

    let git_number: i32 = meta[0].parse()?;
    let git_filename = meta[1];

    return Ok((git_number.into(), git_filename.to_string(), git_data));
}

impl GitObjectAttributes for TreeObject {
    fn from_git_object(git_object: &super::GitObject) -> Result<Box<Self>> {

        // Initializes Variables
        let data = git_object.get_data_as_string()?;
        let (obj_type, obj_size, obj_data) = get_type_size_and_data(&data)?;

        let string_data = String::from_utf8_lossy(&obj_data);

        // let re = Regex::new("[0-7]{5,6} .+?\0.{20}")?;
        let re = Regex::new("[\\d]{5,6} .+?\0.+?(?=[\\d]{5,6}|$)")?;
        let results: Vec<(i32, String, Vec<u8>)> = re
            .find_iter(&string_data)
            .map(|v| {
                let new_v = v.unwrap().as_str();
                println!("{:?}", new_v.splitn(2, "\0").collect::<Vec<&str>>()[1].bytes().len());
                let v = get_number_filename_and_data(
                    new_v
                    ).unwrap();
                println!("{:?}", v);
                return v;
            })
            .collect()
            ;

        let misc_vec_of_vecs: Vec<&Vec<u8>> = results.iter()
            .map(|v| {
                let (_, _, vector) = v;
                for entry in vector {
                    print!("{:x}", entry);
                }
                println!(" - {:?}", String::from_utf8_lossy(vector));
                return vector;
            })
            .collect();

        return Ok(Box::new(Self::new()));
    }
}
