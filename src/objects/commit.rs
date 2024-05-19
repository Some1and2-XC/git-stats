use anyhow::{
    anyhow,
    bail,
    Result,
};

use crate::object::{
    GitObject,
    GitObjectAttributes,
};

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
    /// # use git_stats::objects::commit::CommitObject;
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
    /// # use git_stats::object::{GitObject, GitObjectAttributes};
    /// # use git_stats::objects::commit::CommitObject;
    /// # use git_stats::Repo;
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
