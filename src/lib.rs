#![warn(missing_docs)]

//! git-stats is a library for parsing git files
//! It gives access to information such as what is included in commits and how large the parsed
//! files are.
//! ```
//! # use git_stats::Repo;
//! # use git_stats::objects::commit::CommitObject;
//! # fn main() -> anyhow::Result<()> {
//! let repo = Repo::from_path(".")? // Gets repo object
//!     .enumerate_branches()?; // Makes repo object aware of its branches
//!
//! let main_branch = repo.get_branch("main")?; // Gets the main branch
//! # return Ok(());
//! # }
//! ```

/// The object module is for utilities that relate to git objects. This includes the
/// [`objects::commit::CommitObject`] and more.
pub mod objects;

/// The macro module is for the macros included in this library. This includes
/// the [`macros::ok_or_continue`] macro.
pub mod macros;

pub mod packfiles;

mod repo;

pub use crate::repo::Repo;


const GIT_FOLDERNAME: &'static str = ".git";
