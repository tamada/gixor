//! Git bridge module that abstracts over different Git backends.
//! Provides functions to interact with Git repositories
//! using either the `gix` crate or system Git commands
//! based on feature flags.
use crate::Result;
use crate::repos::Boilerplate;
use std::path::Path;

/// The configuration the clones of the boilerplate repositories are checked out with: the line
/// endings are kept as the repository has them, whatever the platform or the user's git
/// configuration says. `core.autocrlf=true`, the default of Git for Windows, would otherwise turn
/// them into CRLF, so that `dump` would write different files on different platforms. The clones
/// are gixor's own and never edited, so nothing is lost by not converting them.
const LINE_ENDINGS: [&str; 2] = ["core.autocrlf=false", "core.eol=lf"];

#[cfg(feature = "usegix")]
#[path = "gitbridge/gix.rs"]
mod gitctrl;

#[cfg(not(feature = "usegix"))]
#[path = "gitbridge/systemgit.rs"]
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
