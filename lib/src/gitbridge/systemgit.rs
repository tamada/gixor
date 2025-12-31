//! Git operations using the system git command.
//! This module provides functions to interact with Git repositories
//! using the system's Git command-line tool.
//! It includes functions to clone repositories, pull updates, and retrieve commit hashes.
use std::{path::Path, process::Command};

use crate::Result;
use crate::repos::Boilerplate;

pub fn pull(repo_path: &Path, remote: &str, branch: &str) -> Result<()> {
    let r = Command::new("git")
        .args(["pull", remote, branch])
        .current_dir(repo_path)
        .output();
    match r {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                Err(crate::GixorError::Git(format!(
                    "Git command failed: {}",
                    err_msg
                )))
            }
        },
        Err(e) => Err(crate::GixorError::IO(e)),
    }
}

pub fn clone<S: AsRef<str>, P: AsRef<Path>>(url: S, dest_path: P) -> crate::Result<()> {
    let r = Command::new("git")
        .args(["clone", url.as_ref(), dest_path.as_ref().to_str().unwrap()])
        .output();
    match r {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                Err(crate::GixorError::Git(format!(
                    "Git command failed: {}",
                    err_msg
                )))
            }
        },
        Err(e) => Err(crate::GixorError::IO(e)),
    }    
}

/// Returns the latest commit hash (as bytes) of the given boilerplate in the repository located at base_path.
pub fn hash<P: AsRef<Path>>(boilerplate: &Boilerplate, base_path: P) -> Result<Vec<u8>> {
    let base_path = base_path.as_ref();
    log::info!("hash(base_path: {:?}, target: {:?})", base_path, boilerplate.path());
    let r = Command::new("git")
        .args(["log", "--format=%H", "-n", "1", boilerplate.path().to_str().unwrap()])
        .current_dir(boilerplate.repository_path(base_path))
        .output();
    match r {
        Ok(output) => {
            if output.status.success() {
                let hash_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let hash_bytes = hex::decode(hash_str)
                    .map_err(|e| crate::GixorError::Git(format!("Failed to decode hash: {}", e)))?;
                Ok(hash_bytes)
            } else {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                Err(crate::GixorError::Git(format!(
                    "Git command failed: {}",
                    err_msg
                )))
            }
        },
        Err(e) => Err(crate::GixorError::IO(e)),
    }
}

