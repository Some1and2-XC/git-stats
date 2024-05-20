use anyhow::Result;
use regex::Regex;
use super::{
    GitObjectAttributes,
    get_type_size_and_data,
};

/// Object that represents a commit
/// Designed to be initialized using the [`CommitObject::from_str`] function.
#[derive(Debug, Clone)]
pub struct TreeObject {}

impl TreeObject {
    fn new() -> Self {
        return Self {};
    }

    fn from_str() -> Result<Self> {
        todo!()
    }
}

impl GitObjectAttributes for TreeObject {
    fn from_git_object(git_object: &super::GitObject) -> Result<Box<Self>> {
        let data = git_object.get_data_as_string()?;
        let (obj_type, obj_size, obj_data) = get_type_size_and_data(&data)?;

        let string_data = String::from_utf8_lossy(&obj_data);

        println!("{:?}", obj_data);
        println!("{:?}", string_data);

        let split_data = string_data.split(&(100644.to_string() + " ")).collect::<Vec<&str>>();

        println!("{:?}", split_data);
        println!("{:?}", split_data.iter().map(|v| v.len()).collect::<Vec<usize>>());

        return Ok(Box::new(Self::new()));
    }
}
