
use core::fmt;
use std::{
    ffi::OsString, fs, path::PathBuf,
};
use anyhow::{anyhow, bail, ensure, Context, Error, Result};

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
    /// The commit variant
    Commit(CommitObject),
}

/// Trait for every kind of git database object
pub trait GitObjectAttributes {
    /// Gets an object from a passed git object
    /// Usage can be found here [`CommitObject::from_git_object`]
    fn from_git_object(git_object: &GitObject) -> Result<Box<Self>>;
}

/// Object that represents a commit
/// Designed to be initialized using the [`CommitObject::from_str`] function.
#[derive(Debug, Clone)]
pub struct CommitObject {
    /// The hash that points to the commits tree object
    pub tree: String,
    /// The hash that points to the previous commit object
    pub parent: String,
    /// The commit's author string
    pub author: String,
    /// The commit's committer string
    pub committer: String,
}

impl CommitObject {
    /// Does parsing from a string and returns object instance
    /// <section class="warning">
    /// Note raw git files use the <code>\0</code> character to separate metadata from
    /// the actual data so if reading manually, this has to be separated out.
    /// </section>
    ///
    /// ```
    /// # use crate::git_stats::objects::CommitObject;
    /// let commit = CommitObject::from_str("
    /// tree some_big_hash
    /// parent some_big_hash
    /// author some_committer
    /// committer some_committer
    /// ".trim()).unwrap();
    /// assert_eq!(commit.tree, "some_big_hash");
    /// assert_eq!(commit.committer, "some_committer");
    /// ```
    pub fn from_str(in_string: &str) -> Result<Self> {

        let mut tree: Result<String> = Err(anyhow!("Failed to parse 'tree' from string: '{:?}'.", in_string));
        let mut parent: Result<String> = Err(anyhow!("Failed to parse 'parent' from string: '{:?}'.", in_string));
        let mut author: Result<String> = Err(anyhow!("Failed to parse 'author' from string: '{:?}'.", in_string));
        let mut committer: Result<String> = Err(anyhow!("Failed to parse 'committer' from string: '{:?}'.", in_string));

        for v in in_string.split("\n") {
            let v = v.splitn(2, " ").collect::<Box<[&str]>>();
            if v.len() == 2 {
                if v[0] == "tree" {
                    tree = Ok(v[1].to_owned());
                } else if v[0] == "parent" {
                    parent = Ok(v[1].to_owned());
                } else if v[0] == "author" {
                    author = Ok(v[1].to_owned());
                } else if v[0] == "committer" {
                    committer = Ok(v[1].to_owned());
                }
            }
        }

        return Ok(Self {
            tree: tree?,
            parent: parent?,
            author: author?,
            committer: committer?,
        });
    }

}

impl GitObjectAttributes for CommitObject {
    /// Makes a commit object from a filesystem git object
    /// ```
    /// # use crate::git_stats::objects::{CommitObject, GitObject, GitObjectAttributes};
    /// # use crate::git_stats::Repo;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # let repo = Repo::from_path(".")?;
    /// # let some_git_object = GitObject::from_index(&repo, &repo.get_branch_index("main")?)?;
    /// /* get some commit hash */
    /// let obj: CommitObject = *CommitObject::from_git_object(&some_git_object)?;
    /// # return Ok(());
    /// # }
    /// ```
    fn from_git_object(git_object: &GitObject) -> Result<Box<Self>> {

        let inner_data = git_object.get_data()?;
        let in_string = String::from_utf8_lossy(&inner_data).to_string();
        let push_string = in_string.splitn(2, "\0").collect::<Vec<&str>>();
        if push_string.len() != 2 {
            bail!(anyhow!("Couldn't find null character in string: '{}'", in_string));
        }
        let commit_object = Self::from_str(push_string[1]);
        return Ok(Box::new(commit_object?));
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
                CommitObject::from_str(&git_data[1])?
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
