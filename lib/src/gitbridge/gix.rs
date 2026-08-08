//! Git operations using gix crate.
//! This module provides functions to interact with Git repositories
//! using the `gix` crate.
//! It includes functions to clone repositories, pull updates, and retrieve commit hashes.
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gix::{
    remote::{fetch::Outcome, ref_map::Options},
    ObjectId, Repository, Tree,
};

use crate::{repos::Boilerplate, Error, Result};

pub fn clone<S: AsRef<str>, P: AsRef<Path>>(url: S, path: P) -> crate::Result<()> {
    let url = url.as_ref();
    let path = path.as_ref();
    std::fs::create_dir_all(path).map_err(crate::Error::IO)?;
    let url = gix::url::parse(url.as_ref())
        .map_err(|e| crate::Error::Git(format!("Failed to parse URL: {e}")))?;
    let mut prepare_clone = gix::prepare_clone(url.clone(), path)
        .map_err(|e| crate::Error::Git(format!("Failed to prepare clone: {e}")))?;
    log::info!("Cloning {:?} into {path:?}...", url.to_string());
    let (mut prepare_checkout, _) = prepare_clone
        .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| crate::Error::Git(format!("Failed to fetch and checkout: {e}")))?;
    log::info!(
        "Checking out into {} ...",
        prepare_checkout
            .repo()
            .workdir()
            .expect("should be there")
            .display()
    );
    let (repo, _) = prepare_checkout
        .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| crate::Error::Git(format!("Failed to checkout main worktree: {e}")))?;
    log::info!(
        "Repo cloned into {}",
        repo.workdir().expect("directory pre-created").display()
    );
    let remote = repo
        .find_default_remote(gix::remote::Direction::Fetch)
        .unwrap()
        .map_err(|e| crate::Error::Git(format!("Failed to find default remote: {e}")))?;

    log::info!(
        "Default remote: {} -> {}",
        remote
            .name()
            .expect("default remote is always named")
            .as_bstr(),
        remote
            .url(gix::remote::Direction::Fetch)
            .expect("should be the remote URL")
            .to_bstring(),
    );
    Ok(())
}

/// Returns the id of the object that `tree` holds at `path`, or `None` if `path` is absent.
fn entry_id(tree: &Tree<'_>, path: &Path) -> Result<Option<ObjectId>> {
    tree.lookup_entry_by_path(path)
        .map(|entry| entry.map(|e| e.object_id()))
        .map_err(|e| {
            Error::Git(format!(
                "{}: failed to look up the path: {e}",
                path.display()
            ))
        })
}

/// Returns the tree of the commit denoted by `id`.
fn commit_tree(repo: &Repository, id: ObjectId) -> Result<Tree<'_>> {
    repo.find_commit(id)
        .map_err(|e| Error::Git(format!("{id}: failed to find the commit: {e}")))?
        .tree()
        .map_err(|e| Error::Git(format!("{id}: failed to find the tree: {e}")))
}

/// Returns the latest commit hash (as bytes) that changed the given boilerplate,
/// which is the equivalent of `git log --format=%H -n 1 -- {boilerplate.path()}`.
pub fn hash<P: AsRef<Path>>(boilerplate: &Boilerplate, base_path: P) -> Result<Vec<u8>> {
    let repo_path = boilerplate.repo_path(base_path);
    let target = boilerplate.path();
    log::debug!("try to open the git repository: {}", repo_path.display());
    let mut gitrepo = match gix::open(&repo_path) {
        Ok(repo) => Ok(repo),
        Err(_) => {
            let message = format!("{}: Failed to open the repository", repo_path.display());
            log::error!("{message}");
            Err(Error::Git(message.as_str().into()))
        }
    }?;
    // walking by commit time looks up each commit twice, so give the odb a cache.
    gitrepo.object_cache_size_if_unset(4 * 1024 * 1024);

    let mut current = gitrepo
        .head_id()
        .map_err(|e| Error::Git(format!("Failed to get the HEAD: {e}")))?
        .detach();
    let not_found = || {
        Error::Git(format!(
            "{}: no commit found for the path",
            target.display()
        ))
    };
    loop {
        let commit = gitrepo
            .find_commit(current)
            .map_err(|e| Error::Git(format!("{current}: failed to find the commit: {e}")))?;
        let tree = commit
            .tree()
            .map_err(|e| Error::Git(format!("{current}: failed to find the tree: {e}")))?;
        let entry = entry_id(&tree, target)?;

        // Walk down to the first parent holding the very same object at `target`: the path
        // is untouched here, and Git prunes the remaining parents (its TREESAME rule). Doing
        // so matters on merges that dropped a side branch's edit to `target`, since that
        // edit never reached HEAD and must not be reported.
        let mut treesame = None;
        for parent in commit.parent_ids() {
            let parent = parent.detach();
            if entry_id(&commit_tree(&gitrepo, parent)?, target)? == entry {
                treesame = Some(parent);
                break;
            }
        }
        match treesame {
            Some(parent) => current = parent,
            // differs from every parent, so this commit is the one that changed `target`.
            // With no parent at all we are at a root commit that introduced it.
            None if entry.is_some() => return Ok(current.as_bytes().to_vec()),
            None => return Err(not_found()),
        }
    }
}

/// The message of an error together with the ones beneath it.
///
/// gix nests what actually went wrong several levels down, and Display shows only the outermost
/// line, which says that something failed without saying what.
fn chain(e: &dyn std::error::Error) -> String {
    let mut message = e.to_string();
    let mut source = e.source();
    while let Some(e) = source {
        message.push_str(": ");
        message.push_str(&e.to_string());
        source = e.source();
    }
    message
}

fn do_fetch(repo: &gix::Repository, remote: &str) -> Result<Outcome> {
    use gix::{progress::Discard, remote::Direction::Fetch};
    log::info!("Fetching from remote: {remote}");

    let remote = repo
        .find_remote(remote)
        .map_err(|e| Error::Git(format!("Failed to find remote: {e}")))?;
    let c = remote
        .connect(Fetch)
        .map_err(|e| Error::Git(format!("Failed to connect to remote: {e}")))?;
    let r = c
        .prepare_fetch(
            Discard,
            Options {
                prefix_from_spec_as_filter_on_remote: false,
                extra_refspecs: vec![],
                handshake_parameters: vec![],
            },
        )
        .map_err(|e| Error::Git(format!("Failed to prepare fetch: {e}")))?;
    let outcome = r
        .receive(Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| Error::Git(format!("Failed to receive fetch: {}", chain(&e))))?;
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
    merge_base.map(|id| &id == local_id).unwrap_or(false)
}

fn find_merge_strategy<'a>(
    repo: &'a Repository,
    remote: &str,
    branch: &str,
) -> Result<(gix::Id<'a>, gix::Id<'a>, Strategy)> {
    log::debug!("find_merge_strategy: {remote} {branch}");
    let remote_tracking_name = format!("refs/remotes/{remote}/{branch}");
    let remote_tracking = repo
        .find_reference(&remote_tracking_name)
        .map_err(|e| Error::Git(format!("Failed to find remote tracking branch: {e}")))?;
    let local_ref_name = format!("refs/heads/{}", branch.trim_start_matches("refs/heads/"));
    let local_ref = repo
        .find_reference(&local_ref_name)
        .map_err(|e| Error::Git(format!("Failed to find local branch: {e}")))?;
    let local_id = local_ref
        .try_id()
        .ok_or(Error::Git("Failed to get local commit ID".into()))?;
    let remote_id = remote_tracking
        .try_id()
        .ok_or(Error::Git("Failed to get local commit ID".into()))?;
    if local_id == remote_id {
        Ok((local_id, remote_id, Strategy::NoNeed))
    } else if can_fast_forward(repo, &local_id, &remote_id) {
        Ok((local_id, remote_id, Strategy::FastForward))
    } else {
        Ok((local_id, remote_id, Strategy::Merge))
    }
}

/// Removes `path` from the working tree, along with the parent directories it leaves empty.
fn remove_from_worktree(workdir: &Path, path: &Path) {
    log::debug!("removing {}", path.display());
    if let Err(e) = std::fs::remove_file(path) {
        log::warn!("{}: failed to remove: {e}", path.display());
    }
    let mut dir = path.parent();
    while let Some(d) = dir {
        if d == workdir || std::fs::remove_dir(d).is_err() {
            break;
        }
        dir = d.parent();
    }
}

/// Rebuilds the index and the working tree so that both match `commit_id`.
///
/// This is the equivalent of `git reset --hard`. Discarding the working tree wholesale is safe
/// here because gixor keeps its repositories as read-only mirrors of the boilerplate providers:
/// nothing ever writes to them, so there is no local change to preserve. gix offers no API to
/// update an existing working tree, only to populate an empty one, so the tree is written out
/// from the new index and the files that vanished upstream are removed by hand.
fn reset_worktree(repo: &Repository, commit_id: ObjectId) -> Result<()> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| Error::Git("Cannot update the working tree of a bare repository".into()))?
        .to_path_buf();
    let tree_id = commit_tree(repo, commit_id)?.id;

    let old_index = repo
        .index_or_empty()
        .map_err(|e| Error::Git(format!("Failed to read the index: {e}")))?;
    let old_paths = old_index
        .entries()
        .iter()
        .map(|e| gix::path::from_bstr(e.path(&old_index)).into_owned())
        .collect::<HashSet<PathBuf>>();
    drop(old_index);

    let mut index = repo
        .index_from_tree(&tree_id)
        .map_err(|e| Error::Git(format!("{tree_id}: failed to build an index: {e}")))?;
    let new_paths = index
        .entries()
        .iter()
        .map(|e| gix::path::from_bstr(e.path(&index)).into_owned())
        .collect::<HashSet<PathBuf>>();

    for gone in old_paths.difference(&new_paths) {
        remove_from_worktree(&workdir, &workdir.join(gone));
    }

    let mut opts = repo
        .checkout_options(gix::worktree::stack::state::attributes::Source::IdMapping)
        .map_err(|e| Error::Git(format!("Failed to build the checkout options: {e}")))?;
    opts.destination_is_initially_empty = false;
    opts.overwrite_existing = true;

    let outcome = gix::worktree::state::checkout(
        &mut index,
        &workdir,
        repo.objects
            .clone()
            .into_arc()
            .map_err(|e| Error::Git(format!("Failed to share the object database: {e}")))?,
        &gix::progress::Discard,
        &gix::progress::Discard,
        &gix::interrupt::IS_INTERRUPTED,
        opts,
    )
    .map_err(|e| Error::Git(format!("Failed to check out {tree_id}: {e}")))?;
    log::info!(
        "Checked out {} files into {}",
        outcome.files_updated,
        workdir.display()
    );

    index
        .write(Default::default())
        .map_err(|e| Error::Git(format!("Failed to write the index: {e}")))
}

fn fast_forward(repo: &Repository, current_branch: &str, remote_id: gix::Id) -> Result<()> {
    let local_ref_name = format!(
        "refs/heads/{}",
        current_branch.trim_start_matches("refs/heads/")
    );
    log::debug!("Fast-forwarding branch {current_branch} to {remote_id}");
    let mut local_ref = repo
        .find_reference(&local_ref_name)
        .map_err(|e| Error::Git(format!("Failed to find local branch: {e}")))?;
    // The working tree is updated before the branch on purpose. Moving the branch alone leaves
    // the boilerplate files on disk at the old revision, and gixor reads them from there, so
    // the two have to agree. Should we be interrupted in between, leaving the branch behind
    // means the next update fast-forwards again and repairs the repository, whereas moving the
    // branch first would make the update look done and strand the stale files.
    reset_worktree(repo, remote_id.detach())?;
    local_ref
        .set_target_id(remote_id, "Fast-forward")
        .map_err(|e| Error::Git(format!("Failed to fast-forward local branch: {e}")))?;
    log::debug!("Fast-forwarded to {remote_id}");
    Ok(())
}

fn do_merge(repo: &mut Repository, remote: &str, branch: &str) -> Result<()> {
    let (_local_id, remote_id, strategy) = find_merge_strategy(repo, remote, branch)?;
    if strategy == Strategy::NoNeed {
        log::info!("Already up to date.");
        Ok(())
    } else if strategy == Strategy::FastForward {
        log::info!("Fast-forwarding...");
        fast_forward(repo, branch, remote_id)
    } else if strategy == Strategy::Merge {
        log::info!("Merging...");
        Err(Error::Git("Merge commit is not supported yet".into()))
    } else {
        Err(Error::Git("Unknown merge strategy".into()))
    }
}

/// Names gixor as the committer when the machine has no identity of its own.
///
/// gix writes a reflog entry for every reference a fetch moves, and a reflog entry needs a
/// committer. Somewhere `user.name` has never been set — a fresh container, a CI runner — the
/// fetch fails outright with "reflog messages need a committer which isn't set". Asking someone
/// who only wants the boilerplates to introduce themselves first is no way to behave, so gixor
/// signs for them. An identity that is configured is left alone and used as it stands.
fn name_the_committer(repo: &mut Repository) {
    if repo.committer().is_some() {
        return;
    }
    log::debug!("no committer is configured; signing the reflog as gixor");
    let mut config = gix::config::File::new(gix::config::file::Metadata::api());
    let fallbacks = [
        (&gix::config::tree::gitoxide::Committer::NAME_FALLBACK, "gixor"),
        (
            &gix::config::tree::gitoxide::Committer::EMAIL_FALLBACK,
            "gixor@users.noreply.github.com",
        ),
    ];
    for (key, value) in fallbacks {
        if let Err(e) = config.set_raw_value(key, value) {
            log::warn!("could not name the committer: {e}");
            return;
        }
    }
    repo.config_snapshot_mut().append(config);
}

pub fn pull(path: &Path, remote: &str, branch: &str) -> Result<()> {
    let mut repo =
        gix::open(path).map_err(|e| Error::Git(format!("Failed to open repository: {e}")))?;
    name_the_committer(&mut repo);
    let _fetch_outcome = do_fetch(&repo, remote)?;
    do_merge(&mut repo, remote, branch)
}

#[cfg(test)]
mod tests {
    use crate::repos::Repository;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "gixor")
            .env("GIT_AUTHOR_EMAIL", "gixor@example.com")
            .env("GIT_COMMITTER_NAME", "gixor")
            .env("GIT_COMMITTER_EMAIL", "gixor@example.com")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {:?}: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn commit(dir: &Path, message: &str, date: &str) -> String {
        git(dir, &["add", "-A"]);
        let out = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "gixor")
            .env("GIT_AUTHOR_EMAIL", "gixor@example.com")
            .env("GIT_COMMITTER_NAME", "gixor")
            .env("GIT_COMMITTER_EMAIL", "gixor@example.com")
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git commit: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        git(dir, &["rev-parse", "HEAD"])
    }

    fn repository() -> Repository {
        Repository {
            name: "test".to_string(),
            url: "https://github.com/github/gitignore.git".to_string(),
            owner: "github".to_string(),
            repo_name: "gitignore".to_string(),
            path: PathBuf::from("repo"),
        }
    }

    fn hash_of(repo: &Repository, base: &Path, name: &str) -> String {
        let boilerplate = repo
            .iter(base)
            .find(|b| b.path() == Path::new(name))
            .unwrap_or_else(|| panic!("{name}: not found"));
        super::hash(&boilerplate, base)
            .unwrap()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// `hash` must name the commit that last changed the given file, not the repository HEAD,
    /// and it must ignore edits on merged-away branches the way `git log -n 1 -- <path>` does.
    #[test]
    fn test_hash_is_per_file_and_respects_merge_pruning() {
        let base = tempfile::tempdir().unwrap();
        let base = base.path();
        let dir = base.join("repo");
        std::fs::create_dir_all(&dir).unwrap();

        git(&dir, &["init", "-b", "main"]);
        std::fs::write(dir.join("Foo.gitignore"), "a\n").unwrap();
        std::fs::write(dir.join("Other.gitignore"), "other\n").unwrap();
        commit(&dir, "a", "2024-01-01T00:00:00+0000");

        // a side branch edits Foo.gitignore *later* than main does
        git(&dir, &["switch", "-c", "side"]);
        std::fs::write(dir.join("Foo.gitignore"), "c\n").unwrap();
        commit(&dir, "c", "2024-03-01T00:00:00+0000");

        git(&dir, &["switch", "main"]);
        std::fs::write(dir.join("Foo.gitignore"), "b\n").unwrap();
        let expected_foo = commit(&dir, "b", "2024-02-01T00:00:00+0000");

        // the merge discards the side branch's edit, so it never reaches HEAD
        git(&dir, &["merge", "-s", "ours", "--no-edit", "side"]);
        std::fs::write(dir.join("Other.gitignore"), "changed\n").unwrap();
        let expected_other = commit(&dir, "other", "2024-04-01T00:00:00+0000");

        let repo = repository();
        assert_eq!(hash_of(&repo, base, "Foo.gitignore"), expected_foo);
        assert_eq!(hash_of(&repo, base, "Other.gitignore"), expected_other);
        assert_ne!(expected_foo, git(&dir, &["rev-parse", "HEAD"]));
    }

    /// `pull` must leave the working tree at the fetched revision, since gixor reads the
    /// boilerplates from disk rather than from the object database.
    #[test]
    fn test_pull_updates_the_working_tree() {
        let base = tempfile::tempdir().unwrap();
        let remote = base.path().join("remote");
        let work = base.path().join("work");
        std::fs::create_dir_all(remote.join("Global")).unwrap();

        git(&remote, &["init", "-b", "main"]);
        std::fs::write(remote.join("Foo.gitignore"), "v1\n").unwrap();
        std::fs::write(remote.join("Global/Bar.gitignore"), "bar\n").unwrap();
        commit(&remote, "v1", "2024-01-01T00:00:00+0000");

        super::clone(remote.to_string_lossy(), &work).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("Foo.gitignore")).unwrap(),
            "v1\n"
        );

        // modify one boilerplate, drop another one, and add a third
        std::fs::write(remote.join("Foo.gitignore"), "v2\n").unwrap();
        std::fs::remove_file(remote.join("Global/Bar.gitignore")).unwrap();
        std::fs::write(remote.join("Baz.gitignore"), "baz\n").unwrap();
        commit(&remote, "v2", "2024-02-01T00:00:00+0000");

        super::pull(&work, "origin", "main").unwrap();

        assert_eq!(
            std::fs::read_to_string(work.join("Foo.gitignore")).unwrap(),
            "v2\n"
        );
        assert_eq!(
            std::fs::read_to_string(work.join("Baz.gitignore")).unwrap(),
            "baz\n"
        );
        assert!(!work.join("Global/Bar.gitignore").exists());
        assert!(
            !work.join("Global").exists(),
            "emptied directory is left behind"
        );
        // the index has to follow along, or the repository looks dirty to the developer
        assert_eq!(git(&work, &["status", "--short"]), "");
    }

    // These clone into a temporary directory rather than into the repository. Nothing here
    // needs a committed fixture, and the directory goes away on its own even when the test
    // fails partway through.
    #[test]
    fn test_clone_https() {
        let dir = tempfile::tempdir().unwrap();
        let url = "https://github.com/github/gitignore.git";
        super::clone(url, dir.path().join("gitignore-https")).unwrap();
    }

    /// Ignored because it needs an SSH key the remote accepts, which CI has no way to hold.
    /// Run it with `cargo test -- --ignored` where such a key is configured.
    #[ignore = "needs an SSH key for github.com"]
    #[test]
    fn test_clone_ssh() {
        let dir = tempfile::tempdir().unwrap();
        let url = "git@github.com:github/gitignore.git";
        super::clone(url, dir.path().join("gitignore-ssh")).unwrap();
    }
}
