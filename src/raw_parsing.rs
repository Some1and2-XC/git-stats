use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};

use super::GIT_FOLDERNAME;


#[derive(Debug)]
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


pub fn get_objects(repo: PathBuf) -> Vec<GitObject> {
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



// Lists the objects in a git directory
// pub fn get_objects(path: p
