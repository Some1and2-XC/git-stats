use crate::objects::get_type_size_and_data;

use super::{
    GitObject,
    GitObjectAttributes,
};

use anyhow::Result;

/// Object that represents a blob.
#[derive(Debug, Clone)]
pub struct BlobObject {
    /// The data inside the blob, usually utf-8 if type is 100644
    pub data: Vec<u8>,
    /// The size of the blob
    pub size: i32,
}

impl std::fmt::Display for BlobObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return self.to_string().fmt(f);
    }
}

impl BlobObject {
    /// Creates a new BlobObject.
    /// ```
    /// # use git_stats::objects::blob::BlobObject;
    /// let obj = BlobObject::new(vec![1, 2, 3], 999);
    /// assert_eq!(obj.size, 999);
    /// assert_eq!(obj.data, vec![1, 2, 3]);
    /// ```
    pub fn new(data: Vec<u8>, size: i32) -> Self {
        return Self {
            data,
            size,
        };
    }

    /// Creates a string out of the blob data
    /// Uses the [`String::from_utf8_lossy`] so data may be lost
    pub fn to_string(&self) -> String {
        return String::from_utf8_lossy(&self.data).to_string();
    }
}

impl GitObjectAttributes for BlobObject{
    fn from_git_object(git_object: &GitObject) -> Result<Box<Self>> {
        let (obj_type, obj_size, obj_data) = get_type_size_and_data(&git_object.get_data_as_string()?)?;
        assert_eq!(obj_type, "blob");

        return Ok(Box::new(Self::new(
            obj_data,
            obj_size,
        )));
    }
}
