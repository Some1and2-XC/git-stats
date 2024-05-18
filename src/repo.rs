use anyhow::{anyhow, Context,Result};

use super::GIT_FOLDERNAME;

use core::fmt;
use std::{ffi::OsString, fs, path::PathBuf, str::FromStr};

use crate::objects::{CommitObject, GitObjectAttributes, GitObject};

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
    pub dir: PathBuf,
    /// Is None is the branches haven't been searched for yet
    /// Is Some(\[Branches\]) if is has.
    pub branches: Option<Box<[OsString]>>,
}

impl Repo {
    /// Constructs a repo object from a path.
    /// ```
    /// # use crate::git_stats::Repo;
    /// let repo = Repo::from_path(".").unwrap();
    /// ```
    pub fn from_path(path: &str) -> Result<Self> {
        return Self::from_pathbuf(&PathBuf::from_str(path)?);
    }

    /// Tries to construct a repo from a path.
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

    pub fn get_commit_from_index(&self, index: &str) -> Result<CommitObject> {
        let git_object = GitObject::from_index(self, index)?;

        let git_data_string = String::from_utf8_lossy(
            git_object
                .get_data()?
                .as_slice()
            ).replace("\0", "\n");

        return Ok(CommitObject::from_str(&git_data_string)?);
    }

    /// Returns a vec of all the git objects in a git directory
    pub fn get_all_objects(&self) -> Result<Vec<GitObject>> {
        let mut objects: Vec<GitObject> = Vec::new();
        let objects_path = self.dir
            .join("objects")
            ;

        let obj_folder_names = fs::read_dir(objects_path.to_owned())?
            .filter_map(|v| {
                match v {
                    Ok(v) => {
                        let path = v.path();

                        let string_value = path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        if path.is_dir() && string_value.len() == 2 {
                            return Some(path);
                        } else {
                            return None;
                        }

                    },
                    _ => None,
                }
            })
            .collect::<Vec<PathBuf>>();

        let _ = obj_folder_names
            .iter()
            .map(|sub_folder| {
                let sub_folder_name = sub_folder
                    .file_name()
                    .unwrap();
                let files = fs::read_dir(sub_folder)
                    .unwrap()
                    .map(|v| {
                        let path = v
                            .unwrap()
                            .path();
                        let data = fs::read(&path).unwrap();
                        return GitObject::new(
                            sub_folder_name.to_owned(),
                            path.file_name().unwrap().to_owned(),
                            path,
                            data,
                            );
                    })
                    .collect::<Vec<GitObject>>()
                    ;
                objects.extend(files);
                return 0;
            })
            .collect::<Vec<usize>>()
            ;

        return Ok(objects);
    }

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

    pub fn get_branch_index(&self, branch_name: &str) -> Result<String> {
        let branch_path = self.dir
            .join("refs")
            .join("heads")
            .join(branch_name);

        let branch_string = fs::read(branch_path)?;

        let out_string = String::from_utf8(branch_string)?;

        return Ok(out_string.trim().into());
    }

    pub fn get_branch(&self, branch_name: &str) -> Result<CommitObject> {
        let branch_index = self.get_branch_index(branch_name)?;
        let git_object = GitObject::from_index(self, &branch_index)?;
        let commit_object = CommitObject::from_git_object(&git_object)?;
        return Ok(*commit_object);
    }
}
