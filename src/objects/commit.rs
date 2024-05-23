use std::{borrow::Cow, collections::HashMap, str::FromStr};

use anyhow::{
    anyhow, ensure, Context, Result
};

use regex::{Captures, Regex};

use crate::Repo;

use super::{
    blob::BlobObject,
    tree::TreeObject,
    get_type_size_and_data, GitObject, GitObjectAttributes, GitObjectType
};

/// Object that represents a commit
/// Designed to be initialized using the [`CommitObject::from_str`] function.
#[derive(Debug, Clone)]
pub struct CommitObject {
    /// The hash that points to the commits tree object
    pub tree: String,
    /// The hash that points to the previous commit object
    pub parent: Option<String>,
    /// The commit's author string
    pub author: CommitAuthor,
    /// The commit's committer string
    pub committer: CommitAuthor,
    /// The size of the commit object (according to meta data)
    pub size: i32,
    /// The oid of the commit object (according to meta data.)
    pub oid: String,
    /// The message of the commit object.
    pub message: String,
}

impl CommitObject {
    /// Does parsing from a string and returns object instance
    /// <section class="warning">
    /// Note raw git files use the <code>\0</code> character to separate metadata from
    /// the actual data so if reading manually, this has to be separated out.
    /// The size is intended to be read from there
    /// </section>
    ///
    /// ```
    /// # use git_stats::objects::commit::CommitObject;
    /// let commit = CommitObject::from_str("
    /// tree some_big_hash
    /// parent some_big_hash
    /// author MT <some@email.tld> 999999 -0123
    /// committer MT <some@email.tld> 999999 -0123
    ///
    /// Some message
    /// ".trim(), 9999, "some_sha1_hash".into()).unwrap();
    /// assert_eq!(commit.tree, "some_big_hash");
    /// assert_eq!(commit.committer.name, "MT");
    /// ```
    pub fn from_str(in_string: &str, size: i32, oid: String) -> Result<Self> {

        fn get_utf8_from_match_group(capture: &Captures, name: &str) -> String {
            return capture.name(name).unwrap().as_str().into();
        }

        println!("{in_string}");

        let re = Regex::new(&[
            r"tree (?<tree>.+?)\n",
            r"(parent (?<parent>.+?)\n)?",
            r"author (?<author>.+?)\n",
            r"committer (?<committer>.+?)\n",
            r"\n(?<message>.+)",
        ].join("")).unwrap();

        let capture = match re.captures(in_string) {
            Some(v) => v,
            None => return Err(anyhow!("Failed to parse commit from object: '{}'.", oid)),
        };

        let tree = get_utf8_from_match_group(&capture, "tree");
        let parent = match &capture.name("parent") {
            Some(v) => Some(v.as_str().to_string()),
            None => None,
        };
        let author = CommitAuthor::from_string(
            &get_utf8_from_match_group(&capture, "author"))?;
        let committer = CommitAuthor::from_string(
            &get_utf8_from_match_group(&capture, "committer"))?;
        let message = get_utf8_from_match_group(&capture, "message");

        return Ok(Self {
            tree,
            parent,
            author,
            committer,
            size,
            oid,
            message,
        });
    }

    /// Creates commit object from oid and repo
    /// ```
    /// # use git_stats::objects::commit::CommitObject;
    /// # use git_stats::Repo;
    /// let repo = Repo::from_path(".").unwrap();
    /// let commit_oid = repo.get_branch_oid("main").unwrap();
    /// let commit = CommitObject::from_oid(&repo, &commit_oid).unwrap();
    /// ```
    pub fn from_oid(repo: &Repo, oid: &str) -> Result<Self> {
        return Ok(
            *Self::from_git_object(
                &GitObject::from_oid(repo, oid)?
        )?);
    }

    /// Creates a tree from the repos tree attribute.
    pub fn get_tree(&self, repo: &Repo) -> Result<TreeObject> {
        return TreeObject::from_oid(repo, &self.tree);
    }
}

impl GitObjectAttributes for CommitObject {
    /// Makes a commit object from a filesystem git object
    /// ```
    /// # use git_stats::objects::{GitObject, GitObjectAttributes, commit::CommitObject};
    /// # use git_stats::Repo;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # let repo = Repo::from_path(".")?;
    /// # let some_git_object = GitObject::from_oid(&repo, &repo.get_branch_oid("main")?)?;
    /// /* get some commit hash */
    /// let obj: CommitObject = *CommitObject::from_git_object(&some_git_object)?;
    /// # return Ok(());
    /// # }
    /// ```
    fn from_git_object(git_object: &GitObject) -> Result<Box<Self>> {

        let in_string = git_object.get_data_as_string()?;
        let (git_data_type, git_data_size, git_data) = get_type_size_and_data(&in_string)?;

        ensure!(&git_data_type == "commit", anyhow!("Attempted to make commit object out of '{}'", git_data_type));

        let commit_object = Self::from_str(
            &String::from_utf8_lossy(&git_data).to_string(),
            git_data_size,
            git_object.oid.to_owned(),
        )?;
        return Ok(Box::new(commit_object));
    }

    fn get_oid(&self) -> Cow<str> {
        return (&self.oid).into();
    }
}

/// Struct that repesents the author of a commit or the committer.
#[derive(Clone, Debug)]
pub struct CommitAuthor {
    /// This is the same name as shown in github.
    pub name: String,
    /// This is the email of the committer.
    /// This is an option as this is sometimes not included.
    pub email: Option<String>,
    /// This is the timestamp of the commit.
    pub timestamp: u64,
    /// This is the flag that represents the kind.
    pub kind: String,
}

impl CommitAuthor {
    /// Initializes the CommitAuthor
    /// ```
    /// # use git_stats::objects::commit::CommitAuthor;
    /// let in_string = "MT <some@email.tld> 999999 -0123";
    /// let author = CommitAuthor::from_string(in_string).unwrap();
    /// assert_eq!(author.name, "MT");
    /// assert_eq!(author.email.unwrap(), "some@email.tld");
    /// assert_eq!(author.timestamp, 999999);
    /// assert_eq!(author.kind, "0123");
    /// ```
    pub fn from_string(in_str: &str) -> Result<Self> {
        let re = Regex::new(&[
            r"(?<name>.+?) ",
            r"(<(?<email>.+?)> )?",
            r"(?<timestamp>\d+?) ",
            r"-(?<kind>\d{4})",
        ].join("")).unwrap();

        let capture = match re.captures(in_str) {
            Some(v) => v,
            None => return Err(anyhow!("Failed to parse author from string: '{}'.", in_str)),
        };

        let email = match capture.name("email") {
            Some(v) => Some(v.as_str().to_string()),
            None => None,
        };

        return Ok(Self {
            name: capture.name("name").unwrap().as_str().into(),
            email,
            timestamp: capture.name("timestamp").unwrap().as_str().parse().unwrap(),
            kind: capture.name("kind").unwrap().as_str().into(),
        });
    }
}
