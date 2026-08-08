//! Where the boilerplates come from.
//!
//! Two implementations answer the same four questions, and the feature flags pick one of them,
//! the same way [`crate::gitbridge`] picks a Git implementation. `local` keeps clones on the
//! file system and asks Git about them; `embedded` reads a snapshot compiled into the binary,
//! which is what lets the library be built for a target that has neither.
use std::path::Path;

use crate::repos::{Boilerplate, Repository};
use crate::Result;

#[cfg(feature = "embedded")]
#[path = "source/embedded.rs"]
mod imp;

#[cfg(not(feature = "embedded"))]
#[path = "source/local.rs"]
mod imp;

/// Lists the boilerplates the repository holds.
pub(crate) fn list<'a, P: AsRef<Path>>(repo: &'a Repository, base_path: P) -> Vec<Boilerplate<'a>> {
    imp::list(repo, base_path.as_ref())
}

/// Returns the content of the boilerplate file.
pub(crate) fn read<P: AsRef<Path>>(boilerplate: &Boilerplate, base_path: P) -> Result<String> {
    imp::read(boilerplate, base_path.as_ref())
}

/// Returns the commit (as bytes) the content of the boilerplate is to be attributed to.
pub(crate) fn hash<P: AsRef<Path>>(boilerplate: &Boilerplate, base_path: P) -> Result<Vec<u8>> {
    imp::hash(boilerplate, base_path.as_ref())
}

/// Makes the boilerplates of the repository available.
pub(crate) fn prepare<P: AsRef<Path>>(repo: &Repository, base_path: P) -> Result<()> {
    imp::prepare(repo, base_path.as_ref())
}

/// The repositories the boilerplates were taken from, for the configuration to start from.
/// Only the embedded snapshot knows them; elsewhere the configuration file says.
#[cfg(feature = "embedded")]
pub(crate) fn repositories() -> Vec<Repository> {
    imp::repositories()
}
