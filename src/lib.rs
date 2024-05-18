#![warn(missing_docs)]
//! git-stats is a library for parsing git files
//! It gives access to information such as what is included in commits and how large the parsed
//! files are


pub mod objects;
pub mod repo;

const GIT_FOLDERNAME: &'static str = ".git";
