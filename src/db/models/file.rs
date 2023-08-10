use async_graphql::{ComplexObject, SimpleObject};
use opendal::Operator;
use serde::{Deserialize, Serialize};

use std::{io::prelude::*, path::Path};

use crate::constants::CDN_PATH;

#[derive(Debug, Clone, SimpleObject, Deserialize, Serialize)]
#[graphql(complex)]
pub struct MaybeEmptyFile {
    pub id: Option<String>,
}

#[derive(Debug, Clone, SimpleObject, Deserialize, Serialize)]
pub struct FileList {
    pub files: Option<Vec<MaybeEmptyFile>>,
}

pub enum FileStatus {
    Empty,
    Exists,
    Invalid,
}

impl FileStatus {
    pub fn updatable(&self) -> bool {
        match self {
            FileStatus::Empty | FileStatus::Exists => true,
            FileStatus::Invalid => false,
        }
    }

    pub fn insertable(&self) -> bool {
        match self {
            FileStatus::Exists => true,
            FileStatus::Invalid | FileStatus::Empty => false,
        }
    }
}

#[ComplexObject]
impl MaybeEmptyFile {
    /// Returns path that includes the CDN prefix
    async fn absolute_path(&self) -> Option<String> {
        if let Some(id) = &self.id {
            return Some(format!("{}/{}", CDN_PATH, id));
        }
        None
    }
}

impl From<Option<String>> for MaybeEmptyFile {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(path) => MaybeEmptyFile::new(path),
            None => MaybeEmptyFile::empty(),
        }
    }
}

impl From<Option<Vec<String>>> for FileList {
    fn from(value: Option<Vec<String>>) -> Self {
        match value {
            Some(ids) => FileList::new(ids),
            None => FileList::empty(),
        }
    }
}

impl MaybeEmptyFile {
    pub fn new(id: String) -> Self {
        if id.is_empty() {
            return Self::empty();
        }
        Self { id: Some(id) }
    }

    pub fn empty() -> Self {
        Self { id: None }
    }

    pub async fn status(&self) -> anyhow::Result<FileStatus> {
        if let Some(id) = &self.id {
            let loc = &format!("./data/{}", id);
            let path = Path::new(&loc);
            let exists = tokio::fs::metadata(path).await.map(|m| m.is_file())?;

            return match exists {
                true => Ok(FileStatus::Exists),
                false => Ok(FileStatus::Invalid),
            };
        } else {
            return Ok(FileStatus::Empty);
        }
    }

    pub async fn save(&self, fp: &mut std::fs::File, op: &Operator) -> anyhow::Result<()> {
        if let Some(id) = &self.id {
            let mut buffer = Vec::with_capacity(fp.metadata()?.len() as usize);
            fp.read_to_end(&mut buffer)?;
            op.write(id, buffer).await?;
            return Ok(());
        }
        Err(anyhow::Error::msg("File is empty"))
    }
}

impl FileList {
    pub fn new(ids: Vec<String>) -> Self {
        if ids.len() == 0 {
            return Self::empty();
        }
        let files = ids.into_iter().map(MaybeEmptyFile::new).collect();
        Self { files: Some(files) }
    }

    pub fn empty() -> Self {
        Self { files: None }
    }

    pub fn ids(&self) -> Option<Vec<String>> {
        let files = &self.files;
        let x = files.as_ref().map(|x| {
            let mut v = Vec::with_capacity(x.len());
            for i in x {
                if let Some(id) = &i.id {
                    v.push(id.clone());
                }
            }
            v
        });
        x
    }
}
