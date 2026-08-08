//! Boilerplates read from clones on the file system, kept up to date through Git.
use std::path::Path;

use crate::gitbridge;
use crate::repos::{Boilerplate, Repository};
use crate::{Error, Result};

pub(super) fn list<'a>(repo: &'a Repository, base_path: &Path) -> Vec<Boilerplate<'a>> {
    let repo_path = repo.path(base_path);
    ignore::WalkBuilder::new(repo_path.clone())
        .standard_filters(true)
        .build()
        .flatten()
        .map(|entry| entry.into_path())
        .filter(|p| is_gitignore_file(p.file_name()))
        .map(|path| {
            let path = path.strip_prefix(&repo_path).unwrap().to_path_buf();
            Boilerplate::new(
                path.file_stem().unwrap().to_string_lossy().to_string(),
                path,
                repo,
            )
        })
        .collect()
}

pub(super) fn read(boilerplate: &Boilerplate, base_path: &Path) -> Result<String> {
    std::fs::read_to_string(boilerplate.file_path(base_path)).map_err(Error::IO)
}

pub(super) fn hash(boilerplate: &Boilerplate, base_path: &Path) -> Result<Vec<u8>> {
    gitbridge::hash(boilerplate, base_path)
}

/// Clones the repository, or pulls it when it is already there.
pub(super) fn prepare(repo: &Repository, base_path: &Path) -> Result<()> {
    let path = repo.path(base_path);
    if path.join(".git").exists() {
        log::info!("Pulling {} to {}", repo.url, path.display());
        gitbridge::pull(&path, "origin", "main")
    } else {
        log::info!("Cloning {} to {}", repo.url, path.display());
        gitbridge::clone(&repo.url, &path)
    }
}

fn is_gitignore_file(name: Option<&std::ffi::OsStr>) -> bool {
    name.and_then(|n| n.to_str())
        .is_some_and(|name| name.ends_with(".gitignore"))
}
