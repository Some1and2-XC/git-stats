/// The commit module is for holding the [`commit::CommitObject`] struct.
pub mod commit;

/// The tree module is for holding the [`tree::TreeObject`] struct.
pub mod tree;

use std::fs;
use anyhow::{anyhow, ensure, Result};

use crate::repo::Repo;

extern crate miniz_oxide;

/// Utility function for getting metadata about an input
/// ```
/// # use git_stats::objects::get_type_size_and_data;
/// let in_data = "commit 999\0somedata"; // This is how git objects are layed out
/// let (obj_type, obj_size, obj_data) = get_type_size_and_data(in_data).unwrap();
/// assert_eq!(&obj_type, "commit");
/// assert_eq!(obj_size, 999);
/// assert_eq!(obj_data, "somedata".bytes().collect::<Vec<u8>>());
/// ```
pub fn get_type_size_and_data(in_str: &str) -> Result<(String, i32, Vec<u8>)> {

    let split_data = in_str
        .splitn(2, "\0")
        .map(|v| v)
        .collect::<Vec<&str>>();

    ensure!(split_data.len() == 2, anyhow!(
        "Null character not found! Data: {}",
        in_str,
    ));

    let git_data: Vec<u8> = split_data[1].bytes().collect::<Vec<u8>>();

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

    let git_data_type = meta[0];
    let git_data_size: i32 = meta[1].parse()?;

    return Ok((git_data_type.into(), git_data_size, git_data));
}

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


/*
#[derive(Debug, Clone)]
pub struct TreeObject {
}

#[derive(Debug, Clone)]
pub struct BlobObject {
}
*/

/// Struct representing an object in the git database.
/// Use [`GitObject::new`] to create a new instance.
#[derive(Debug, Clone)]
pub struct GitObject {
    /// The folder + filename of the object
    pub oid: String,
    /// The data inside the object
    pub data: Vec<u8>,
}

impl GitObject {
    /// Creates a new [`GitObject`]
    /// ```
    /// # use git_stats::objects::GitObject;
    /// let oid = "some_oid";
    /// let data = "some_data".bytes().collect::<Vec<u8>>();
    /// let git_object = GitObject::new(oid.to_string(), data);
    /// assert_eq!(git_object.oid, "some_oid");
    /// assert_eq!(git_object.data, "some_data".bytes().collect::<Vec<u8>>());
    /// ```
    pub fn new(oid: String, data: Vec<u8>) -> Self {
        GitObject {
            oid,
            data,
        }
    }

    /// Creates a dummy instance of a git object.
    /// Designed to be used for testing only.
    /// ```
    /// # use git_stats::objects::{GitObject, GitObjectType::Commit};
    /// let git_object = GitObject::new_dummy_commit();
    /// let commit_obj = match git_object.initialize_from_data().unwrap() {
    ///     Commit(obj) => obj,
    ///     _ => panic!(),
    /// };
    /// assert_eq!(commit_obj.size, 999);
    /// assert_eq!(&commit_obj.parent.unwrap(), "some_hash");
    /// assert_eq!(&commit_obj.author, "some_hash");
    /// assert_eq!(&commit_obj.committer, "some_hash");
    /// ```
    pub fn new_dummy_commit() -> Self {
        let some_commit = vec![
            "commit 999\0tree some_hash",
            "parent some_hash",
            "author some_hash",
            "committer some_hash",
        ].join("\n");

        return GitObject::new(
            "some_oid".to_string(),
            miniz_oxide::deflate::compress_to_vec_zlib(&some_commit.bytes().collect::<Vec<u8>>(), 0),
        );
    }

    /// Initializes a some git object variant from data.
    /// Uses [`GitObject::new_dummy_commit`] for its dummy data.
    /// ```
    /// # use git_stats::objects::{GitObject, GitObjectType};
    /// let git_object = GitObject::new_dummy_commit();
    /// let commit = match git_object.initialize_from_data().unwrap() {
    ///     GitObjectType::Commit(obj) => obj,
    ///     _ => panic!(),
    /// };
    /// assert_eq!(commit.size, 999);
    /// assert_eq!(&commit.parent.unwrap(), "some_hash");
    /// ```
    /// This example also shows the data that is being used.
    /// The `compress_to_vec_zlib()` function is from [`miniz_oxide::deflate::compress_to_vec_zlib`].
    /// ```
    /// # use git_stats::objects::{GitObject, GitObjectType};
    /// # use miniz_oxide::deflate::compress_to_vec_zlib;
    /// let some_commit = "
    /// commit 999\0tree some_hash
    /// parent some_hash
    /// author some_hash
    /// committer some_hash
    /// ".trim();
    ///
    /// let git_object = GitObject::new(
    ///     "some_oid".to_string(),
    ///     compress_to_vec_zlib(&some_commit.bytes().collect::<Vec<u8>>(), 0),
    /// );
    ///
    /// let commit = match git_object.initialize_from_data().unwrap() {
    ///     GitObjectType::Commit(obj) => obj,
    ///     _ => panic!("This should be a commit!"),
    /// };
    /// assert_eq!(commit.size, 999);
    /// assert_eq!(&commit.parent.unwrap(), "some_hash");
    /// ```
    pub fn initialize_from_data(&self) -> Result<GitObjectType> {

        let string_data = self.get_data_as_string()?;
        let (git_data_type, git_data_size, git_data) = get_type_size_and_data(&string_data)?;

        if git_data_type == "commit" {
            return Ok(GitObjectType::Commit(
                commit::CommitObject::from_str(&String::from_utf8_lossy(&git_data).to_string(), git_data_size)?
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

    /// Returns the inner data as a string
    pub fn get_data_as_string(&self) -> Result<String> {
        return Ok(String::from_utf8_lossy(&self.get_data()?).to_string());
    }

    /// Initializes GitObject from an oid
    pub fn from_oid(repo: &Repo, oid: &str) -> Result<Self> {
        if oid.len() < 3 {
            return Err(anyhow!("Git object index must have longer hash than 3, index: '{}'", oid));
        }

        let (sub_folder, filename) = oid.split_at(2);

        let object_path = repo.dir
            .join("objects")
            .join(sub_folder)
            .join(filename)
            ;

        let data = fs::read(&object_path)?;

        return Ok(GitObject::new(
            sub_folder.to_owned() + filename,
            data,
        ));
    }

    /// Gets and decompresses the underlying data
    /// from the object
    pub fn get_data(&self) -> Result<Vec<u8>> {
        match miniz_oxide::inflate::decompress_to_vec_zlib(&self.data) {
            Ok(v) => Ok(v),
            Err(_) => Err(anyhow!("Can't decompress data from object '{:?}' with data: '{:?}'!", self.oid, self.data)),
        }
    }

    /// Gets the sha1 hash of the inner data
    pub fn get_hash(&self) -> Result<String> {

        // Checks if the data can even be decompressed
        let hash_data = self.get_data()?;

        // Gets hash
        return Ok(sha1_smol::Sha1::from(hash_data).digest().to_string());
    }
}
