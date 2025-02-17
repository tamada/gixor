//! This module contains the function of `git pull` command.
//! The `pull` function is used to fetch and integrate with another repository or a local branch.
//! This function is sed in [`gita`](https://docs.rs/gita/latest/gita/) crate.
//! This crate is requires the old version of [`git2`](https://docs.rs/git2/0.20.0/git2/index.html) crate.
//! Therefore, I just copy the code from `gita` and update some parts.

// ------------- the following part is copied from gita crate ------------------
/*
 * libgit2 "pull" example - shows how to pull remote data into a local branch.
 *
 * Written by the libgit2 contributors
 *
 * To the extent possible under law, the author(s) have dedicated all copyright
 * and related and neighboring rights to this software to the public domain
 * worldwide. This software is distributed without any warranty.
 *
 * You should have received a copy of the CC0 Public Domain Dedication along
 * with this software. If not, see
 * <http://creativecommons.org/publicdomain/zero/1.0/>.
 */
use git2::Repository;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    str,
};

fn do_fetch<'a>(
    repo: &'a git2::Repository,
    refs: &[&str],
    remote: &'a mut git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>, git2::Error> {
    let mut cb = git2::RemoteCallbacks::new();

    // Print out our transfer progress.
    cb.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                "Resolving deltas {}/{}\r",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            print!(
                "Received {}/{} objects ({}) in {} bytes\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes()
            );
        }
        io::stdout().flush().unwrap();
        true
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb);
    // Always fetch all tags.
    // Perform a download and also update tips
    fo.download_tags(git2::AutotagOption::All);
    log::info!("Fetching {} for repo", remote.name().unwrap());
    remote.fetch(refs, Some(&mut fo), None)?;

    // If there are local objects (we got a thin pack), then tell the user
    // how many objects we saved from having to cross the network.
    let stats = remote.stats();
    if stats.local_objects() > 0 {
        log::info!(
            "\rReceived {}/{} objects in {} bytes (used {} local objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        log::info!(
            "\rReceived {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    repo.reference_to_annotated_commit(&fetch_head)
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    log::info!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        log::info!("Merge conficts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

fn do_merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<(), git2::Error> {
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    // 2. Do the appopriate merge
    if analysis.0.is_fast_forward() {
        log::info!("Doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(repo, &head_commit, &fetch_commit)?;
    } else {
        log::info!("Nothing to do...");
    }
    Ok(())
}

/// `git pull`
pub fn pull(repo: &Path, remote: &str, branch: &str) -> Result<(), git2::Error> {
    let git_repo = Repository::open(repo)?;
    let mut remote_branch = git_repo.find_remote(remote)?;
    let fetch_commit = do_fetch(&git_repo, &[branch], &mut remote_branch)?;
    do_merge(&git_repo, remote, fetch_commit)
}
// ------------- the above part is copied from gita crate ------------------

use git2::{Cred, RemoteCallbacks};

pub fn clone<S: AsRef<str>, P: AsRef<Path>>(url: S, path: P) -> crate::Result<()> {
    let url = url.as_ref();
    let path = path.as_ref();
    if url.starts_with("https://") {
        clone_with_https(url, path)
    } else if url.starts_with("ssh://") || url.starts_with("git@") {
        clone_with_ssh(url, path)
    } else {
        Err(crate::GixorError::Fatal(format!(
            "{}: Unsupported protocol",
            url
        )))
    }
}

fn clone_with_https<S: AsRef<str>, P: AsRef<Path>>(url: S, path: P) -> crate::Result<()> {
    match git2::Repository::clone(url.as_ref(), path.as_ref()) {
        Err(e) => Err(crate::GixorError::Git(e)),
        Ok(_) => Ok(()),
    }
}

fn find_privatekey() -> crate::Result<PathBuf> {
    let ssh_dir = dirs::home_dir().unwrap().join(".ssh");

    let path_rsa = ssh_dir.join("id_rsa");
    let path_ed25519 = ssh_dir.join("id_ed25519");
    if path_rsa.exists() {
        Ok(path_rsa)
    } else if path_ed25519.exists() {
        Ok(path_ed25519)
    } else {
        Err(crate::GixorError::Fatal("No private key found".to_string()))
    }
}

/// Clone a repository with SSH protocol. This code is copied from Sample code of RepoBuilder in git2 crate.
/// https://docs.rs/git2/latest/git2/build/struct.RepoBuilder.html#example
fn clone_with_ssh<S: AsRef<str>, P: AsRef<Path>>(url: S, path: P) -> crate::Result<()> {
    // Prepare callbacks.
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        let privatekey = find_privatekey().unwrap();
        Cred::ssh_key(username_from_url.unwrap(), None, privatekey.as_path(), None)
    });

    // Prepare fetch options.
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    // Prepare builder.
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    // Clone the project.
    match builder.clone(url.as_ref(), path.as_ref()) {
        Err(e) => Err(crate::GixorError::Git(e)),
        Ok(_) => Ok(()),
    }
}
