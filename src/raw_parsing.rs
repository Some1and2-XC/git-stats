
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
extern crate sha1_smol;

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
