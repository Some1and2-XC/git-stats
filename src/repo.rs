use super::GIT_FOLDERNAME;

use anyhow::{anyhow, ensure, Result};

use core::fmt;
use std::{
    ffi::OsString, fs,
    path::PathBuf, str::FromStr,
};

use crate::objects::{
    GitObjectAttributes, GitObject,
    commit::CommitObject,
};

use crate::macros::ok_or_continue;

// Defines Repo Parsing Error
#[derive(Debug, Clone)]
pub struct ParseGitRepoError;

impl fmt::Display for ParseGitRepoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "Can't parse git object from input!");
    }
}

impl std::error::Error for ParseGitRepoError {}

/// Struct that represents a repository.
#[derive(Debug, Clone)]
pub struct Repo {
    /// The directory the repo is in.
    pub dir: PathBuf,
    /// To use this attribute, using [`Repo::enumerate_branches`] is required.
    /// Is None is the branches haven't been searched for yet.
    /// This attribute is type `Some([Branches])` if is has.
    pub branches: Option<Box<[OsString]>>,
}

impl Repo {
    /// Constructs a repo from a path.
    /// ```
    /// # use crate::git_stats::Repo;
    /// let repo = Repo::from_path(".").unwrap();
    /// ```
    pub fn from_path(path: &str) -> Result<Self> {
        return Self::from_pathbuf(&PathBuf::from_str(path)?);
    }

    /// Constructs a repo from a PathBuf object.
    /// ```
    /// # use crate::git_stats::Repo;
    /// # use std::{path::PathBuf, str::FromStr};
    /// let path = PathBuf::from_str(".").unwrap();
    /// let repo = Repo::from_pathbuf(&path).unwrap();
    /// ```
    pub fn from_pathbuf(path: &PathBuf) -> Result<Self> {

        let git_path = path.join(GIT_FOLDERNAME);
        if git_path.exists() && git_path.is_dir() {
            return Ok(
                Repo {
                    dir: git_path,
                    branches: None,
                });
        } else {
            return Err(anyhow!("Couldn't read repo in path: '{:?}'", git_path));
        }
    }

    /// Gets a commit object using the oid
    /// ```
    /// # use git_stats::Repo;
    /// # let repo = Repo::from_path(".").unwrap();
    /// let head: String = repo.get_branch_oid("main").unwrap(); // gets the OID from the main branch
    /// let commit = repo.get_commit_from_oid(&head).unwrap();
    /// ```
    pub fn get_commit_from_oid(&self, index: &str) -> Result<CommitObject> {
        let git_object = GitObject::from_oid(self, index)?;

        let git_data = String::from_utf8_lossy(
            git_object
                .get_data()?
                .as_slice()
            ).splitn(2, "\0")
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            ;

        ensure!(git_data.len() == 2, anyhow!("Git Data type isn't of length 2! Data: '{:?}'", git_data));

        let data_size = git_data[0]
            .split(" ")
            .nth(1);

        let data_size_int: i32 = match data_size {
            Some(v) => v.parse()?,
            None => return Err(anyhow!("Couldn't parse int from {:?}!", data_size)),
        };

        return Ok(
            CommitObject::from_str(
                &git_data[1],
                data_size_int,
                git_object.oid.to_owned(),
            )?
        );
    }

    /// Returns a vec of all the git objects in a git directory
    pub fn get_all_objects(&self) -> Result<Vec<GitObject>> {
        let mut objects: Vec<GitObject> = Vec::new();
        let objects_path = self.dir
            .join("objects")
            ;

        for folder in fs::read_dir(&objects_path)? {

            let checked_folder = ok_or_continue!(folder);

            // Breaks early if it isn't a directory
            if !checked_folder.path().is_dir() { continue; };
            if checked_folder.file_name().len() != 2 { continue; };

            for file in ok_or_continue!(fs::read_dir(checked_folder.path())) {
                let checked_file = ok_or_continue!(file);

                let oid = (checked_folder.file_name().to_string_lossy() +
                       checked_file.file_name().to_string_lossy()).to_string();

                let data = ok_or_continue!(fs::read(checked_file.path()));

                objects.push(
                    GitObject::new(
                        oid,
                        data,
                    )
                );
            }
        }
        return Ok(objects);
    }

    /// Makes the repo aware of the branches is has
    /// Returns the original instance
    /// ```should_panic
    /// # use anyhow::Result;
    /// # use std::ffi::OsString;
    /// # fn main() -> Result<()> {
    /// # use git_stats::Repo;
    /// let repo = Repo::from_path(".")?;
    /// let branches: Box<[OsString]> = repo.branches.unwrap(); // This unwrap throws
    /// # Ok(()) }
    /// ```
    /// ```
    /// # use git_stats::Repo;
    /// # use anyhow::Result;
    /// # use std::ffi::OsString;
    /// # fn main() -> Result<()> {
    /// let repo = Repo::from_path(".")?.enumerate_branches()?;
    /// let branches: Box<[OsString]> = repo.branches.unwrap(); // This unwrap doesn't throw
    /// # Ok(()) }
    /// ```
    pub fn enumerate_branches(mut self) -> Result<Self> {
        let path = self.dir.join("refs").join("heads");
        self.branches = Some(
            fs::read_dir(&path)?
                .filter_map(
                    |dir| match dir {
                        Ok(v) => Some(v.file_name()),
                        Err(_) => None,
                    }
                ).collect::<Box<[OsString]>>()
            );

        return Ok(self);
    }

    /// Gets a branch from the name of a branch
    /// ```
    /// # use git_stats::Repo;
    /// # use git_stats::objects::GitObject;
    /// # let repo = Repo::from_path(".").unwrap();
    /// /* With some repo named 'repo' */
    /// let branch = repo.get_branch_oid("main").unwrap(); // Gets the oid of the branch
    /// // A new GitObject can be initialized from this oid
    /// let git_object = GitObject::from_oid(&repo, &branch).unwrap();
    /// ```
    pub fn get_branch_oid(&self, branch_name: &str) -> Result<String> {
        let branch_path = self.dir
            .join("refs")
            .join("heads")
            .join(branch_name);

        let branch_string = fs::read(branch_path)?;

        let out_string = String::from_utf8(branch_string)?;

        return Ok(out_string.trim().into());
    }

    /// Gets a commit object by branch name
    /// ```
    /// # use git_stats::Repo;
    /// # let repo = Repo::from_path(".").unwrap();
    /// let head_commit = repo.get_branch("main").unwrap();
    /// ```
    pub fn get_branch(&self, branch_name: &str) -> Result<CommitObject> {
        let branch_index = self.get_branch_oid(branch_name)?;
        let git_object = GitObject::from_oid(self, &branch_index)?;
        let commit_object = CommitObject::from_git_object(&git_object)?;
        return Ok(*commit_object);
    }
}
