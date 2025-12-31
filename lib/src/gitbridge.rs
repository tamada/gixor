//! Git bridge module that abstracts over different Git backends.
//! Provides functions to interact with Git repositories
//! using either the `gix` crate, `git2` crate, or system Git commands
//! based on feature flags.
use std::path::Path;
use crate::Result;
use crate::repos::Boilerplate;

#[cfg_attr(feature="usegix", path="gitbridge/gix.rs")]
#[cfg_attr(feature="uselibgit", path="gitbridge/git2.rs")]
#[cfg_attr(not(any(feature="usegix", feature="uselibgit")), path="gitbridge/systemgit.rs")]
mod gitctrl;

/// Returns the latest commit hash (as bytes) of the repository at the given path.
pub fn pull(repository_path: &Path, remote_name: &str, branch_name: &str) -> Result<()> {
    gitctrl::pull(repository_path, remote_name, branch_name)
}

/// Clones the repository from the given URL to the specified destination path.
pub fn clone<S: AsRef<str>, P: AsRef<Path>>(url: S, dest_path: P) -> Result<()> {
    gitctrl::clone(url, dest_path)
}

/// Returns the latest commit hash (as bytes) of the given boilerplate in the repository located at base_path.
pub fn hash<P: AsRef<Path>>(boilerplate: &Boilerplate, base_path: P) -> Result<Vec<u8>> {
    gitctrl::hash(boilerplate, base_path)
}
