
use core::fmt;
use std::{
    ffi::OsString,
    path::PathBuf,
    error::Error,
    fs,
};
use miniz_oxide::inflate::DecompressError;

use crate::repo::Repo;

extern crate miniz_oxide;

// Defines Git Object Parse Error
#[derive(Debug, Clone)]
pub struct ParseGitObjectError;

impl fmt::Display for ParseGitObjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "Can't parse git object from input!");
    }
}

impl Error for ParseGitObjectError {}

pub enum GitObjectType {
    Commit(CommitObject),
}

#[derive(Debug, Clone)]
pub struct CommitObject {
    pub tree: Option<String>,
    pub parent: Option<String>,
    pub author: Option<String>,
    pub committer: Option<String>,
}

impl CommitObject {
    /// Does parsing from a string and returns object instance
    pub fn from_str(in_string: &str) -> Self {

        let mut obj = CommitObject {
            tree: None,
            parent: None,
            author: None,
            committer: None,
        };

        for v in in_string.split("\n") {
            let v = v.splitn(2, " ").collect::<Box<[&str]>>();
            if v.len() == 2 {
                if v[0] == "tree" {
                    obj.tree = Some(v[1].to_owned());
                } else if v[0] == "parent" {
                    obj.parent = Some(v[1].to_owned());
                } else if v[0] == "author" {
                    obj.author = Some(v[1].to_owned());
                } else if v[0] == "committer" {
                    obj.committer = Some(v[1].to_owned());
                }
            }
        }

        return obj;
    }
}

#[derive(Debug, Clone)]
pub struct GitObject {
    pub dirname: OsString,
    pub filename: OsString,
    pub filepath: PathBuf,
    pub data: Vec<u8>,
}

impl GitObject {
    pub fn new(dirname: OsString, filename: OsString, filepath: PathBuf, data: Vec<u8>) -> Self {
        GitObject {
            dirname,
            filename,
            filepath,
            data,
        }
    }

    /// Initializes GitObject from a git name
    pub fn from_index(repo: &Repo, index: &str) -> Result<Self, ParseGitObjectError> {
        if index.len() < 3 {
            println!("Index less than 3? Path: '{}'", index);
            return Err(ParseGitObjectError);
        }

        let (sub_folder, filename) = index.split_at(2);

        let object_path = repo.dir
            .join("objects")
            .join(sub_folder)
            .join(filename)
            ;

        let data = match fs::read(&object_path) {
            Ok(v) => v,
            Err(_) => {
                println!("Can't read file: '{}'.", object_path.to_string_lossy().to_string());
                println!("Rehashing all objects... (This might take a while)");
                return Err(ParseGitObjectError);
            },
        };

        return Ok(GitObject::new(
            sub_folder.into(),
            filename.into(),
            object_path,
            data,
        ));
    }

    pub fn decompress_data(&self) -> Result<Vec<u8>, DecompressError> {
        miniz_oxide::inflate::decompress_to_vec_zlib(&self.data)
    }

    /// Gets and decompresses the underlying data
    /// from the object
    pub fn get_data(&self) -> Vec<u8> {
        return self.decompress_data().unwrap();
    }

    pub fn get_hash(&self) -> Option<String> {

        // Checks if the data can even be decompressed
        let hash_data = match self.decompress_data() {
            Ok(v) => v,
            Err(_) => return None,
        };

        // Gets hash
        return Some(sha1_smol::Sha1::from(hash_data).digest().to_string());
    }
}
