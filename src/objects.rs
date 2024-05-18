
use core::fmt;
use std::{
    ffi::OsString, fs, path::PathBuf,
};
use anyhow::{anyhow, ensure, Context, Result};

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

impl std::error::Error for ParseGitObjectError {}

#[derive(Debug)]
pub enum GitObjectType {
    Commit(CommitObject),
}

pub trait GitObjectAttributes {
    fn from_git_object(git_object: &GitObject) -> Result<Box<Self>>;
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

impl GitObjectAttributes for CommitObject {
    fn from_git_object(git_object: &GitObject) -> Result<Box<Self>> {
        let inner_data = git_object.get_data()?;
        let in_string = String::from_utf8_lossy(&inner_data).to_string();
        let commit_object = Self::from_str(&in_string);
        return Ok(Box::new(commit_object));
    }
}

#[derive(Debug, Clone)]
pub struct TreeObject {
}

#[derive(Debug, Clone)]
pub struct BlobObject {
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

    pub fn initialize_from_data(&self) -> Result<GitObjectType> {

        let internal_data = self.get_data()?.as_slice().to_owned();
        let git_data = String::from_utf8_lossy(&internal_data)
            .splitn(2, "\0")
            .map(|v| v.to_string())
            .collect::<Vec<String>>();


        ensure!(git_data.len() == 2, anyhow!(
            "Null character not found! Data: {}",
            String::from_utf8_lossy(&internal_data)
        ));

        // Gets first segment
        let git_data_meta = git_data[0]
            .splitn(2, " ")
            .map(|v| v)
            .collect::<Vec<&str>>()
            ;

        ensure!(git_data_meta.len() == 2, anyhow!("Git Data type isn't of length 2! Data: '{:?}'", git_data_meta));

        let git_data_type = git_data_meta[0];
        let git_data_size = git_data_meta[1];

        // println!("Data: {:?}", git_data_type);
        if git_data_type == "tree" {
            println!(
                "Data: '{:?}' & Diff: '{}'",
                internal_data,
                git_data[1].len() - git_data_size.parse::<usize>()?,
                )
        }

        if git_data_type == "commit" {
            return Ok(GitObjectType::Commit(
                CommitObject::from_str(&git_data[1])
            ));
        } else if git_data_type == "tree" {
            // println!("{:?}", git_data);
            return Err(anyhow!("Not implemented yet!"));
        } else if git_data_type == "blob" {
            // println!("{:?}", git_data);
            return Err(anyhow!("Not implemented yet!"));
        } else {
            return Err(anyhow!("Git Datatype: '{}' not found!", git_data_type));
        }
    }

    /// Initializes GitObject from a git name
    pub fn from_index(repo: &Repo, index: &str) -> Result<Self> {
        if index.len() < 3 {
            return Err(anyhow!("Git object index must have longer hash than 3, index: '{}'", index));
        }

        let (sub_folder, filename) = index.split_at(2);

        let object_path = repo.dir
            .join("objects")
            .join(sub_folder)
            .join(filename)
            ;

        let data = fs::read(&object_path)
            .with_context(
                || format!("Failed to read file: '{}'.", object_path.to_string_lossy().to_string())
                )?;

        return Ok(GitObject::new(
            sub_folder.into(),
            filename.into(),
            object_path,
            data,
        ));
    }

    /// Gets and decompresses the underlying data
    /// from the object
    pub fn get_data(&self) -> Result<Vec<u8>> {
        match miniz_oxide::inflate::decompress_to_vec_zlib(&self.data) {
            Ok(v) => Ok(v),
            Err(_) => Err(anyhow!("Can't decompress data from object '{:?}'!", self.filepath)),
        }
    }

    /// Gets the sha1 hash of the object inner
    pub fn get_hash(&self) -> Result<String> {

        // Checks if the data can even be decompressed
        let hash_data = self.get_data()?;

        // Gets hash
        return Ok(sha1_smol::Sha1::from(hash_data).digest().to_string());
    }
}
