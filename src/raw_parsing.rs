use core::fmt;

use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::ptr::hash;

use miniz_oxide::inflate::DecompressError;

use super::GIT_FOLDERNAME;

extern crate miniz_oxide;
extern crate sha1_smol;


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
    pub fn from_index(basedir: &PathBuf, index: &str) -> Result<Self, ParseGitObjectError> {
        if index.len() < 3 {
            return Err(ParseGitObjectError);
        }

        let (sub_folder, filename) = index.split_at(2);

        let object_path = basedir
            .join("objects")
            .join(sub_folder)
            .join(filename)
            ;

        let data = match fs::read(&object_path) {
            Ok(v) => v,
            Err(e) => return Err(ParseGitObjectError),
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

    pub fn get_data(&self) -> String {
        String::from_utf8_lossy(
            &self.decompress_data().unwrap()
            ).to_string()
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

/// Gets a list of the files in a directory
pub fn ls(path: &Path) -> Result<Box<[DirEntry]>, Box<dyn Error>> {
    let paths = fs::read_dir(path)
        .unwrap()
        .into_iter()
        .map(|v| v.unwrap())
        .collect::<Box<[DirEntry]>>()
        ;
    return Ok(paths);
}


pub fn get_repository(files: Box<[DirEntry]>) -> Option<PathBuf> {

    let mut repository: Option<PathBuf> = None;

    for file in files.into_iter() {
        let path = file.path();
        if path.is_dir() && file.file_name() == GIT_FOLDERNAME {
            repository = Some(path);
        }
    }
    return repository;
}


pub fn get_git_objects(repo: PathBuf) -> Vec<GitObject> {
    let mut objects: Vec<GitObject> = Vec::new();
    let objects_path = repo
        .join("objects")
        ;

    let obj_folder_names = fs::read_dir(objects_path.to_owned())
        .unwrap()
        .filter_map(|v| {
            match v {
                Ok(v) => {
                    let path = v.path();

                    let filename_string = path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                        ;

                    if path.is_dir() && filename_string.len() == 2 {
                        return Some(path);
                    } else {
                        return None;
                    }
                }
                _ => None,
            }
        })
        .collect::<Vec<PathBuf>>()
        ;

    let _ = obj_folder_names
        .iter()
        .map(|sub_folder| {
            let sub_folder_name = sub_folder.file_name().unwrap();
            let files = fs::read_dir(sub_folder).unwrap()
                .map(|v| {
                    let path = v.unwrap().path();
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

    return objects;
}

pub fn get_branch(git_dir: &PathBuf, branch_name: String) -> Option<String> {
    let branch_path = git_dir
        .join("refs")
        .join("heads")
        .join(branch_name);

    let branch_string = match fs::read(branch_path) {
        Ok(v) => v,
        Err(_) => return None,
    };

    let out_string = match String::from_utf8(branch_string) {
        Ok(v) => v,
        Err(_) => return None,
    };

    return Some(out_string.trim().into());
}



// Lists the objects in a git directory
// pub fn get_objects(path: p
