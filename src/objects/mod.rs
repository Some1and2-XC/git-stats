pub mod commit;

use core::fmt;
use std::{
    ffi::{CString}, fs,
};
use anyhow::{anyhow, ensure, Result};

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

/// Enum that reprensents a database git object
#[derive(Debug)]
pub enum GitObjectType {
    /// Commit variant
    Commit(commit::CommitObject),
    /// Tree variant
    Tree,
    /// Blob variant
    Blob,
    /// Executable blob variant
    BlobExecutable,
    /// Link variant
    Link,
}

/// Trait for every kind of git database object
pub trait GitObjectAttributes {
    /// Gets an object from a passed git object
    /// Usage can be found here [`commit::CommitObject::from_git_object`]
    fn from_git_object(git_object: &GitObject) -> Result<Box<Self>>;
}


#[derive(Debug, Clone)]
pub struct TreeObject {
}

#[derive(Debug, Clone)]
pub struct BlobObject {
}

#[derive(Debug, Clone)]
pub struct GitObject {
    pub oid: CString,
    pub data: Vec<u8>,
}

impl GitObject {
    pub fn new(oid: CString, data: Vec<u8>) -> Self {
        GitObject {
            oid,
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
                commit::CommitObject::from_str(&git_data[1])?
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

        let data = fs::read(&object_path)?;

        return Ok(GitObject::new(
            CString::new(sub_folder.to_owned() + filename)?,
            data,
        ));
    }

    /// Gets and decompresses the underlying data
    /// from the object
    pub fn get_data(&self) -> Result<Vec<u8>> {
        match miniz_oxide::inflate::decompress_to_vec_zlib(&self.data) {
            Ok(v) => Ok(v),
            Err(_) => Err(anyhow!("Can't decompress data from object '{:?}'!", self.oid)),
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
