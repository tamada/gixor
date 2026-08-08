//! Boilerplates compiled into the binary from the snapshot under `boilerplates/`.
//!
//! Nothing here touches the file system or Git, which is what lets the library be built for a
//! target that offers neither. The snapshot cannot be updated at run time by design: `prepare`
//! has nothing to do, and the permalinks point at the commit the snapshot was taken from.
use std::path::{Path, PathBuf};

use crate::repos::{Boilerplate, Repository};
use crate::{Error, Result};

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

pub(super) fn list<'a>(repo: &'a Repository, _base_path: &Path) -> Vec<Boilerplate<'a>> {
    BOILERPLATES
        .iter()
        .filter(|(name, _, _)| *name == repo.name)
        .map(|(_, path, _)| {
            let path = PathBuf::from(path);
            Boilerplate::new(
                path.file_stem().unwrap().to_string_lossy().to_string(),
                path,
                repo,
            )
        })
        .collect()
}

pub(super) fn read(boilerplate: &Boilerplate, _base_path: &Path) -> Result<String> {
    let wanted = boilerplate.path().to_string_lossy().replace('\\', "/");
    BOILERPLATES
        .iter()
        .find(|(name, path, _)| *name == boilerplate.repository_name() && *path == wanted)
        .map(|(_, _, content)| content.to_string())
        .ok_or_else(|| Error::BoilerplateNotFound(boilerplate.name().to_string()))
}

/// The commit the snapshot was taken from, which is what the permalinks have to name. Every
/// boilerplate of a repository shares it, since the snapshot was taken at a single revision.
pub(super) fn hash(boilerplate: &Boilerplate, _base_path: &Path) -> Result<Vec<u8>> {
    let name = boilerplate.repository_name();
    let commit = REPOSITORIES
        .iter()
        .find(|(repo, _, _, _, _)| *repo == name)
        .map(|(_, _, _, _, commit)| *commit)
        .ok_or_else(|| Error::RepositoryNotFound(name.to_string()))?;
    hex::decode(commit).map_err(|e| Error::Git(format!("{commit}: not a commit hash: {e}")))
}

/// The snapshot is whatever it was when it was built, so there is nothing to fetch.
pub(super) fn prepare(_repo: &Repository, _base_path: &Path) -> Result<()> {
    Ok(())
}

/// The repositories the snapshot was taken from, for a configuration to start from.
pub(super) fn repositories() -> Vec<Repository> {
    REPOSITORIES
        .iter()
        .map(|(name, url, owner, repo_name, _)| Repository {
            name: name.to_string(),
            url: url.to_string(),
            owner: owner.to_string(),
            repo_name: repo_name.to_string(),
            path: PathBuf::from(name),
        })
        .collect()
}
