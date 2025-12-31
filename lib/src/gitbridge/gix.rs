//! Git operations using gix crate.
//! This module provides functions to interact with Git repositories
//! using the `gix` crate.
//! It includes functions to clone repositories, pull updates, and retrieve commit hashes.
//! 
use std::path::Path;

use gix::{Repository, remote::{fetch::Outcome, ref_map::Options}};

use crate::{GixorError, Result, repos::Boilerplate};

pub fn clone<S: AsRef<str>, P: AsRef<Path>>(url: S, path: P) -> crate::Result<()> {
    let url = url.as_ref();
    let path = path.as_ref();
    std::fs::create_dir_all(path).map_err(|e| crate::GixorError::IO(e))?;
    let url = gix::url::parse(url.as_ref())
        .map_err(|e| crate::GixorError::Git(format!("Failed to parse URL: {}", e)))?;
    let mut prepare_clone = gix::prepare_clone(url.clone(), path)
        .map_err(|e| crate::GixorError::Git(format!("Failed to prepare clone: {}", e)))?;
    log::info!("Cloning {:?} into {path:?}...", url.to_string());
    let (mut prepare_checkout, _) =
        prepare_clone.fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
            .map_err(|e| crate::GixorError::Git(format!("Failed to fetch and checkout: {}", e)))?;
    log::info!(
        "Checking out into {} ...",
        prepare_checkout.repo().workdir().expect("should be there").display()
    );
    let (repo, _) = prepare_checkout.main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| crate::GixorError::Git(format!("Failed to checkout main worktree: {}", e)))?;
    log::info!(
        "Repo cloned into {}",
        repo.workdir().expect("directory pre-created").display()
    );
    let remote = repo
        .find_default_remote(gix::remote::Direction::Fetch).unwrap()
        .map_err(|e| crate::GixorError::Git(format!("Failed to find default remote: {}", e)))?;

    log::info!(
        "Default remote: {} -> {}",
        remote.name().expect("default remote is always named").as_bstr(),
        remote
            .url(gix::remote::Direction::Fetch)
            .expect("should be the remote URL")
            .to_bstring(),
    );
    Ok(())
}

pub fn hash<P: AsRef<Path>>(boilerplate: &Boilerplate, base_path: P) -> Result<Vec<u8>> {
    let path = boilerplate.repo_path(base_path);
    log::debug!("try to open the git repository: {}", path.display());
    let gitrepo = match gix::open(&path) {
        Ok(repo) => Ok(repo),
        Err(_) => {
            let message = format!(
                "{}: Failed to open the repository",
                path.display()
            );
            log::error!("{}", message);
            Err(GixorError::Git(message.as_str().into()))
        },
    }?;
    let head = gitrepo.head();
    match head {
        Ok(mut h) => match h.peel_to_commit() {
            Ok(commit) => Ok(commit.id().as_bytes().to_vec()),
            Err(e) => Err(GixorError::Git(format!("Failed to peel to commit: {}", e))),
        },
        Err(_) => Err(GixorError::Git("Failed to get the HEAD".into())),
    }
}

fn do_fetch(repo: &gix::Repository, remote: &str) -> Result<Outcome> {
    use gix::{progress::Discard, remote::Direction::Fetch};
    log::info!("Fetching from remote: {}", remote);

    let remote = repo.find_remote(remote)
        .map_err(|e| GixorError::Git(format!("Failed to find remote: {}", e)))?;
    let c = remote.connect(Fetch)
        .map_err(|e| GixorError::Git(format!("Failed to connect to remote: {}", e)))?;
    let r = c.prepare_fetch(Discard,
        Options{
            prefix_from_spec_as_filter_on_remote: false,
            extra_refspecs: vec![],
            handshake_parameters: vec![] 
        }).map_err(|e| GixorError::Git(format!("Failed to prepare fetch: {}", e)))?;
    let outcome = r.receive(Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| GixorError::Git(format!("Failed to receive fetch: {}", e)))?;
    log::info!("Fetch completed: {:?}", outcome.status);
    Ok(outcome)
}

#[derive(PartialEq)]
enum Strategy {
    FastForward,
    Merge,
    NoNeed,
}

fn can_fast_forward(repo: &Repository, local_id: &gix::Id, remote_id: &gix::Id) -> bool {
    let merge_base = repo.merge_base(*local_id, *remote_id);
    merge_base.map(|id| &id == local_id)
        .unwrap_or(false)
}

fn find_merge_strategy<'a>(repo: &'a Repository, remote: &str, branch: &str) -> Result<(gix::Id<'a>, gix::Id<'a>, Strategy)> {
    log::debug!("find_merge_strategy: {} {}", remote, branch);
    let remote_tracking_name = format!("refs/remotes/{remote}/{branch}");
    let remote_tracking = repo.find_reference(&remote_tracking_name)
        .map_err(|e| GixorError::Git(format!("Failed to find remote tracking branch: {}", e)))?;
    let local_ref_name = format!("refs/heads/{}", branch.trim_start_matches("refs/heads/"));
    let local_ref = repo.find_reference(&local_ref_name)
        .map_err(|e| GixorError::Git(format!("Failed to find local branch: {}", e)))?;
    let local_id = local_ref.try_id().ok_or(GixorError::Git("Failed to get local commit ID".into()))?;
    let remote_id = remote_tracking.try_id().ok_or(GixorError::Git("Failed to get local commit ID".into()))?;
    if local_id == remote_id {
        Ok((local_id, remote_id, Strategy::NoNeed))
    } else if can_fast_forward(repo, &local_id, &remote_id) {
        Ok((local_id, remote_id, Strategy::FastForward))
    } else {
        Ok((local_id, remote_id, Strategy::Merge))
    }
}

fn fast_forward(repo: &Repository, current_branch: &str, remote_id: gix::Id,) -> Result<()> {
    let local_ref_name = format!("refs/heads/{}", current_branch.trim_start_matches("refs/heads/"));
    log::debug!("Fast-forwarding branch {} to {}", current_branch, remote_id);
    let mut local_ref = repo.find_reference(&local_ref_name)
        .map_err(|e| GixorError::Git(format!("Failed to find local branch: {}", e)))?;
    local_ref.set_target_id(remote_id, "Fast-forward")
        .map_err(|e| GixorError::Git(format!("Failed to fast-forward local branch: {}", e)))?;
    log::debug!("Fast-forwarded to {}", remote_id);
    Ok(())
}

fn do_merge(repo: &mut Repository, remote: &str, branch: &str) -> Result<()> {
    let (_local_id, remote_id, strategy) = find_merge_strategy(repo, remote, branch)?;
    if strategy == Strategy::NoNeed {
        log::info!("Already up to date.");
        Ok(())
    } else if strategy == Strategy::FastForward {
        log::info!("Fast-forwarding...");
        fast_forward(&repo, branch, remote_id)
    } else if strategy == Strategy::Merge {
        log::info!("Merging...");
        Err(GixorError::Git("Merge commit is not supported yet".into()))
    } else {
        Err(GixorError::Git("Unknown merge strategy".into()))
    }
}

pub fn pull(path: &Path, remote: &str, branch: &str) -> Result<()> {
    let mut repo = gix::open(path)
        .map_err(|e| GixorError::Git(format!("Failed to open repository: {}", e)))?;
    let _fetch_outcome = do_fetch(&repo, remote)?;
    do_merge(&mut repo, remote, branch)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_clone_https() {
        let url = "https://github.com/github/gitignore.git";
        let path = "../testdata/test-clone-with-gix/gitignore-https";
        super::clone(url, path).unwrap();
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_clone_ssh() {
        let url = "git@github.com:github/gitignore.git";
        let path = "../testdata/test-clone-with-gix/gitignore-ssh";
        super::clone(url, path).unwrap();
        std::fs::remove_dir_all(path).unwrap();
    }

}