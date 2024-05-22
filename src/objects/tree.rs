use anyhow::{anyhow, ensure, Result};
use regex::bytes::Regex;
use super::{
    GitObjectAttributes,
    get_type_size_and_data,
};

use crate::macros::ok_or_continue;


/// Object that represents a Tree
/// Designed to be initialized using the [`TreeObject::from_git_object`] function.
#[derive(Debug, Clone)]
pub struct TreeObject {
    /// A list of the items the tree has
    pub items: Vec<TreeItem>,
    /// The size of the tree object (according to meta data)
    pub size: i32,
}

impl TreeObject {
    pub fn new(items: Vec<TreeItem>, size: i32) -> Self {
        return Self {
            items,
            size,
        };
    }
}

/// Object that represents an item in a tree.
#[derive(Debug, Clone)]
pub struct TreeItem {
    /// The type of item the file is
    /// The control bits are similar to linux fs
    /// 040000: tree
    /// 100644: for a regular file
    /// 100755: executable
    /// 120000: symlink
    /// 160000: gitlink
    mode: i32,
    /// The name of the folder the tree item refers to
    filename: String,
    /// The OID that points to the data the tree item refers to
    oid: String,
}

impl TreeItem {
    pub fn new(mode: i32, filename: String, oid: String) -> Self {
        return Self {
            mode,
            filename,
            oid,
        };
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

        let in_data = git_object.get_data()?;
        let (obj_type, obj_size, _) = get_type_size_and_data(&git_object.get_data_as_string()?)?;

        assert_eq!(obj_type, "tree");

        println!("{:?}", in_data);

        // Initializes Variables
        let re = Regex::new(r"(?<mode>1?[0-7]{5}) (?<filename>.+?)\x00(?<data>(?-u:.){20})").unwrap();

        let results: Vec<TreeItem> = re
            .captures_iter(&in_data)
            .map(|v| {
                // This method uses unwraps because if the following values can't be decoded,
                // that means some logic is critically incorrect (should never happen at all.)
                let (_, value_bytes): (_, [&[u8];3]) = v.extract();

                let number_value: i32 = String::from_utf8(value_bytes[0].to_vec())
                    .unwrap()
                    .parse()
                    .unwrap();

                let filename = String::from_utf8(value_bytes[1].to_vec()).unwrap();

                let oid = value_bytes[2]
                    .iter()
                    .map(|v| format!("{:02x}", v))
                    .collect::<Vec<String>>()
                    .join("")
                    ;

                return TreeItem::new(
                    number_value,
                    filename,
                    oid
                );
            })
            .collect()
            ;

        return Ok(Box::new(Self::new(results, obj_size)));
    }
}
