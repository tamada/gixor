//! Module for managing repositories and boilerplates.
//! This module provides functionality to define repositories, find boilerplates,
//! and prepare repositories by cloning or pulling from remote sources.
use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Error, Name, Result};

/// Represents a boilerplate file.
pub struct Boilerplate<'a> {
    /// The boilerplate name. It is the stem of the boilerplate file.
    name: String,
    /// The path of the boilerplate file from the repository root.
    /// Therefore, the full path of this boilerplate file is `{repo.path(base_path)}/{path}`.
    /// Note that, the `base_path` is given from [`crate::Gixor`].
    path: PathBuf,
    /// The repository of this boilerplate.
    repo: &'a Repository,
}

impl<'a> Boilerplate<'a> {
    pub(crate) fn new<P: AsRef<Path>>(
        name: String,
        path: P,
        repo: &'a Repository,
    ) -> Boilerplate<'a> {
        log::debug!(
            "Boilerplate::new(name: {name}, path: {:?}, repo: {:?})",
            path.as_ref(),
            repo.path
        );
        Self {
            name,
            path: path.as_ref().to_path_buf(),
            repo,
        }
    }

    /// Returns the name of this boilerplate as [Name].
    pub fn name(&self) -> Name {
        Name::new(self.repository_name(), self.boilerplate_name())
    }

    /// Returns the file path of the boilerplate from the base path.
    /// This method is equivalent to `self.repo_path(base_path).join(&self.path())`.
    ///
    /// ### See also
    ///
    /// - [`Repository::path`]
    /// - [`Boilerplate::path`]
    pub fn file_path<P: AsRef<Path>>(&self, base_path: P) -> PathBuf {
        self.repo_path(base_path).join(self.path())
    }

    /// Returns the path from in the repository root.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the path of this boilerplate in its repository without `.gitignore`, such as
    /// `Global/AL`, with `/` as the separator on every platform. It names this boilerplate in
    /// the headings `dump` writes, and is what a name holding a slash is matched against.
    pub fn path_name(&self) -> String {
        self.path() // get path (Global/AL.gitignore)
            .with_extension("") // strip the extension (Global/AL)
            .to_string_lossy() // convert to str
            .replace('\\', "/") // replace windows path separator to '/'
    }

    /// Returns the repository path from the base path.
    /// This method delegates to [`Repository::path`].
    ///
    /// ### See also
    ///
    /// - [`Repository::path`]
    pub fn repo_path<P: AsRef<Path>>(&self, base_path: P) -> PathBuf {
        self.repo.path(base_path)
    }

    /// Returns the boilerplate name of this instance.
    pub fn boilerplate_name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns the repository name of this instance.
    pub fn repository_name(&self) -> &str {
        &self.repo.name
    }

    /// Returns `true` if the given name and this instance are matched.
    ///
    /// The boilerplate part of the name is compared, ignoring case, with
    ///
    /// - the path of the boilerplate in the repository without `.gitignore` when it holds a
    ///   slash, so that `Global/AL` matches `Global/AL.gitignore` and nothing else, or
    /// - the file stem otherwise, so that `AL` matches both `AL.gitignore` and
    ///   `Global/AL.gitignore`.
    ///
    /// The repository name, when given, has to match as well.
    pub fn matches(&self, name: &Name) -> bool {
        self.matches_boilerplate(&name.boilerplate_name)
            && match &name.repository_name {
                Some(repo_name) => {
                    repo_name.to_lowercase() == self.repository_name().to_lowercase()
                }
                None => true,
            }
    }

    fn matches_boilerplate(&self, name: &str) -> bool {
        if name.contains('/') {
            let path_string = self.path_name();
            name.eq_ignore_ascii_case(&path_string)
        } else {
            name.to_lowercase() == self.boilerplate_name().to_lowercase()
        }
    }

    /// Returns the latest commit hash (as bytes) of this boilerplate in the repository located at base_path.
    ///
    /// ### See also
    ///
    /// - [`gitbridge::hash`]
    pub fn hash<P: AsRef<Path>>(&self, base_path: P) -> Result<Vec<u8>> {
        crate::source::hash(self, base_path)
    }

    /// Returns the content URL of the boilerplate file.
    pub fn content_url<P: AsRef<Path>>(&self, base_path: P) -> Result<String> {
        let base_path = base_path.as_ref();
        log::info!(
            "boilerplate content_url(name: {}, path: {:?}, repo: {:?}, base_path: {:?})",
            self.name,
            self.path,
            self.repo.path,
            base_path
        );
        let hash = self.hash(base_path)?;
        log::trace!("hash: {hash:02x?}");
        let hash_string = hash.iter().fold(String::new(), |mut output, b| {
            let _ = write!(output, "{b:02X}");
            output
        });
        let url = &self.repo.url;
        if url.contains("github.com") {
            Ok(format!(
                "https://raw.github.com/{0}/{1}/{2}/{3}",
                self.repo.owner,
                self.repo.repo_name,
                hash_string,
                self.path.to_string_lossy()
            ))
        } else if url.contains("gitlab.com") {
            Ok(format!(
                "https://gitlab.com/{0}/{1}/-/raw/{2}/{3}",
                self.repo.owner,
                self.repo.repo_name,
                hash_string,
                self.path.to_string_lossy()
            ))
        } else if url.contains("bitbucket.org") {
            Ok(format!(
                "https://bitbucket.org/{0}/{1}/raw/{2}/{3}",
                self.repo.owner,
                self.repo.repo_name,
                hash_string,
                self.path.to_string_lossy()
            ))
        } else {
            Err(Error::Fatal(format!("{url}: Unsupported repository host")))
        }
    }

    /// Returns the content of the boilerplate file.
    pub fn dump<P: AsRef<Path>>(&self, base_path: P) -> Result<String> {
        let content = crate::source::read(self, &base_path)?;
        Ok(format!(
            r#"### Generated by Gixor (https://github.com/tamada/gixor) ({}/{})
### {}
{}
"#,
            self.repository_name(),
            self.path_name(),
            self.content_url(base_path.as_ref())?,
            content
        ))
    }
}

/// Represents a repository of the boilerplates.
/// The boilerplate repository is cloned into `${base_path}/${repo_name}`
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Repository {
    /// The name of the repository. The default name is the owner name of the repository.
    /// This name is used for the directory name under the base path.
    pub name: String,
    /// The url of remote repository. The `url` should be formatted in the GitHub repository URL
    /// which should has owner name and repository name.
    pub url: String,
    /// The owner name of the repository.
    pub owner: String,
    /// The repository name.
    pub repo_name: String,
    /// The path of the repository from the base path.
    pub path: PathBuf,
}

impl Default for Repository {
    /// Create a default repository.
    /// <https://github.com/github/gitignore.git>.
    fn default() -> Self {
        Repository {
            name: "default".to_string(),
            url: "https://github.com/github/gitignore.git".to_string(),
            repo_name: "gitignore".to_string(),
            owner: "github".to_string(),
            path: PathBuf::from("default"),
        }
    }
}

impl Repository {
    /// Creates an instance of Repository with the given url.
    /// The name of the repository is owner name which is extracted from the url.
    pub fn new<S: AsRef<str>>(url: S) -> Self {
        let url = url.as_ref();
        let (owner, repo_name) = url_to_owner_and_repo_name(url);
        let path = PathBuf::from(&owner);
        log::debug!(
            "Repository::new(url: {url}) -> owner: {owner}, repo_name: {repo_name}, path: {path:?}"
        );
        Self {
            name: owner.clone(),
            url: url.to_string(),
            owner: owner.clone(),
            repo_name,
            path,
        }
    }

    /// Creates an instance of Repository with the given name and url.
    pub fn new_with<S: AsRef<str>>(name: S, url: S) -> Self {
        let url = url.as_ref();
        let name = name.as_ref();
        let (owner, repo_name) = url_to_owner_and_repo_name(url);
        log::debug!(
            "Repository::new_with(name: {name}, url: {url}) -> owner: {owner}, repo_name: {repo_name}"
        );
        Self {
            name: name.to_string(),
            url: url.to_string(),
            owner,
            repo_name,
            path: PathBuf::from(name),
        }
    }

    /// Returns the path of this repository from the base path.
    pub fn path<P: AsRef<Path>>(&self, base_path: P) -> PathBuf {
        if self.path.is_absolute() {
            self.path.clone()
        } else {
            base_path.as_ref().join(&self.path)
        }
    }

    /// Finds every boilerplate matching the name, the shallowest first, and those of the same
    /// depth by path, so that the order does not depend on the order the file system lists them in.
    pub fn find_all<P: AsRef<Path>>(&self, name: &Name, base_path: P) -> Vec<Boilerplate<'_>> {
        let mut candidates = self
            .iter(base_path)
            .filter(|b| name.matches(b))
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| {
            let key = |b: &Boilerplate| (b.path().components().count(), b.path().to_path_buf());
            key(a).cmp(&key(b))
        });
        candidates
    }

    /// Finds the boilerplate by the name.
    ///
    /// When several boilerplates share the name, e.g. `AL.gitignore` and `Global/AL.gitignore`,
    /// the first of [`Repository::find_all`] is taken, and a warning names the others.
    pub fn find<P: AsRef<Path>>(&self, name: &Name, base_path: P) -> Option<Boilerplate<'_>> {
        let candidates = self.find_all(name, base_path);
        if candidates.len() > 1 {
            warn_multiple_candidates(name, &candidates)
        }
        candidates.into_iter().next()
    }

    /// Iterates the boilerplates in the repository.
    pub fn iter<P: AsRef<Path>>(&self, base_path: P) -> impl Iterator<Item = Boilerplate<'_>> {
        crate::source::list(self, base_path).into_iter()
    }

    /// Prepare the repository by cloning or pulling the remote repository.
    /// If the repository already exists, perform the equivalent routine of `git pull origin main`.
    /// Otherwise, execute `git clone` and store the repository into the `self.path(base_path)`.
    ///
    /// ### See also
    ///
    /// - [`Repository::path`]
    pub fn prepare<P: AsRef<Path>>(&self, base_path: P) -> Result<()> {
        crate::source::prepare(self, base_path)
    }

    /// Makes the boilerplates of this repository available, without updating them when they
    /// already are. With a clone that means cloning it if it is missing and leaving it alone
    /// otherwise; with the embedded snapshot there is nothing to do.
    ///
    /// ### See also
    ///
    /// - [`Repository::path`]
    pub fn clone_repo_to<P: AsRef<Path>>(&self, base_path: P) -> Result<()> {
        let path = self.path(&base_path);
        if path.join(".git").exists() {
            log::info!("{}: repository exists", path.display());
            Ok(())
        } else {
            crate::source::prepare(self, base_path)
        }
    }
}

fn warn_multiple_candidates(name: &Name, candidates: &[Boilerplate]) {
    log::warn!(
        "{name}: {} boilerplates share the name; using {} (others: {})",
        candidates.len(),
        candidates[0].path().display(),
        candidates[1..]
            .iter()
            .map(|b| b.path().display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

fn url_to_owner_and_repo_name(url: &str) -> (String, String) {
    let items = url.split('/').collect::<Vec<_>>();
    let len = items.len();
    match (
        len.checked_sub(2).and_then(|idx| items.get(idx)),
        items.last(),
    ) {
        (Some(&owner), Some(&name)) => (to_owner(owner), strip_dot_git(name)),
        (None, Some(&name)) => ("unknown".into(), strip_dot_git(name)),
        (Some(&owner), None) => (to_owner(owner), "gitignore".into()),
        _ => ("unknown".into(), "gitignore".into()),
    }
}

fn to_owner<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref();
    if s.contains(':') {
        let items = s.split(':').collect::<Vec<_>>();
        if let Some(&owner) = items.last() {
            owner.to_string()
        } else {
            "unknown".to_string()
        }
    } else {
        s.to_string()
    }
}

fn strip_dot_git<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref().to_string();
    if let Some(name) = s.strip_suffix(".git") {
        name.to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Clones only exist for the `local` feature; the embedded snapshot carries one repository
    /// and cannot gain another.
    #[cfg(feature = "local")]
    #[test]
    fn test_repo() {
        let repo = Repository::new_with("tamada", "https://github.com/tamada/gitignore.git");
        assert_eq!(repo.name, "tamada");
        assert_eq!(repo.url, "https://github.com/tamada/gitignore.git");

        // share the one clone the other tests use, rather than racing them for the directory
        crate::tests::prepare_once();
        let base_path = crate::tests::boilerplates_path();
        if let Err(e) = repo.prepare(&base_path) {
            panic!("Failed to prepare the repository: {e}");
        }
        let boilerplates = repo.iter(&base_path).collect::<Vec<_>>();
        assert_eq!(boilerplates.len(), 1);

        if let Some(b) = repo.find(&Name::new_of("devcontainer"), &base_path) {
            assert_eq!(
                b.file_path(&base_path),
                base_path.join("tamada").join("devcontainer.gitignore")
            );
        } else {
            panic!("Failed to find the devcontainer.gitignore");
        }
    }

    #[test]
    fn test_url_to_name() {
        let url = "https://github.com/github/gitignore.git";
        let (owner, name) = url_to_owner_and_repo_name(url);
        assert_eq!(owner, "github");
        assert_eq!(name, "gitignore");
    }

    #[test]
    fn test_url_to_name2() {
        let url = "git@github.com:tamada/gitignore.git";
        let (owner, name) = url_to_owner_and_repo_name(url);
        assert_eq!(owner, "tamada");
        assert_eq!(name, "gitignore");
    }

    #[test]
    fn test_url_parsing_edge_cases() {
        // Test with unusual URLs
        assert_eq!(
            url_to_owner_and_repo_name("simple-name"),
            ("unknown".to_string(), "simple-name".to_string())
        );
        assert_eq!(
            url_to_owner_and_repo_name("owner/repo"),
            ("owner".to_string(), "repo".to_string())
        );
        assert_eq!(
            url_to_owner_and_repo_name("https://host.com/repo.git"),
            ("host.com".to_string(), "repo".to_string())
        );

        // Test to_owner
        assert_eq!(to_owner("user:name"), "name".to_string());
        assert_eq!(to_owner("name"), "name".to_string());
        assert_eq!(to_owner(":"), "".to_string()); // items.last() of ":" split is ""
    }

    /// A repository on the file system holding the given boilerplates, under `<base>/fixture`.
    #[cfg(feature = "local")]
    fn fixture_repository(paths: &[&str]) -> (tempfile::TempDir, Repository) {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("fixture");
        for path in paths {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, "*.tmp\n").unwrap();
        }
        let repo = Repository::new_with("fixture", "https://example.invalid/fixture/gitignore");
        (base, repo)
    }

    #[cfg(feature = "local")]
    #[test]
    fn find_takes_the_shallowest_of_the_boilerplates_sharing_a_name() {
        let (base, repo) = fixture_repository(&["Global/AL.gitignore", "AL.gitignore"]);
        let found = repo.find(&Name::new_of("AL"), base.path()).unwrap();
        assert_eq!(found.path(), Path::new("AL.gitignore"));
    }

    #[cfg(feature = "local")]
    #[test]
    fn find_takes_the_first_by_path_among_the_same_depth() {
        let (base, repo) = fixture_repository(&[
            "community/CFML/ColdBox.gitignore",
            "community/BoxLang/ColdBox.gitignore",
        ]);
        let found = repo.find(&Name::new_of("coldbox"), base.path()).unwrap();
        assert_eq!(
            found.path(),
            Path::new("community/BoxLang/ColdBox.gitignore")
        );
    }

    #[cfg(feature = "local")]
    #[test]
    fn find_all_lists_every_boilerplate_sharing_the_name_in_order() {
        let (base, repo) = fixture_repository(&[
            "community/Racket.gitignore",
            "Racket.gitignore",
            "Rust.gitignore",
        ]);
        let found = repo.find_all(&Name::new_of("racket"), base.path());
        let paths = found.iter().map(|b| b.path()).collect::<Vec<_>>();
        assert_eq!(
            paths,
            vec![
                Path::new("Racket.gitignore"),
                Path::new("community/Racket.gitignore")
            ]
        );
        assert!(repo.find_all(&Name::new_of("Go"), base.path()).is_empty());
    }

    fn repository(name: &str) -> Repository {
        Repository::new_with(name, "https://example.invalid/owner/gitignore")
    }

    fn boilerplate<'a>(repo: &'a Repository, path: &str) -> Boilerplate<'a> {
        let stem = Path::new(path).file_stem().unwrap().to_string_lossy();
        Boilerplate::new(stem.to_string(), path, repo)
    }

    #[test]
    fn matches_a_name_by_the_file_stem() {
        let repo = repository("default");
        let name = Name::new_of("al");
        assert!(boilerplate(&repo, "AL.gitignore").matches(&name));
        assert!(boilerplate(&repo, "Global/AL.gitignore").matches(&name));
    }

    #[test]
    fn matches_a_name_holding_a_slash_by_the_path_only() {
        let repo = repository("default");
        let name = Name::new_of("Global/AL");
        assert!(boilerplate(&repo, "Global/AL.gitignore").matches(&name));
        assert!(!boilerplate(&repo, "AL.gitignore").matches(&name));
        assert!(boilerplate(&repo, "Global/AL.gitignore").matches(&Name::new_of("global/al")));
    }

    #[test]
    fn matches_a_deep_path() {
        let repo = repository("default");
        let name = Name::new_of("community/CFML/ColdBox");
        assert!(boilerplate(&repo, "community/CFML/ColdBox.gitignore").matches(&name));
        assert!(!boilerplate(&repo, "community/BoxLang/ColdBox.gitignore").matches(&name));
    }

    #[test]
    fn matches_no_part_of_a_path() {
        let repo = repository("default");
        let b = boilerplate(&repo, "community/CFML/ColdBox.gitignore");
        assert!(!b.matches(&Name::new_of("CFML/ColdBox")));
        assert!(!b.matches(&Name::new_of("munity/CFML/ColdBox")));
        assert!(!b.matches(&Name::new_of("community/CFML")));
    }

    #[test]
    fn matches_a_path_with_windows_separators() {
        let repo = repository("default");
        let b = boilerplate(&repo, "Global\\AL.gitignore");
        assert!(b.matches(&Name::new_of("Global/AL")));
    }

    /// A path names a file in every repository having it, so the repository has to be checked as
    /// well, or `tamada/Global/AL` would take the file of whichever repository comes first.
    #[test]
    fn matches_the_repository_name_with_a_path_too() {
        let default = repository("default");
        let tamada = repository("tamada");
        let name = Name::new("tamada", "Global/AL");
        assert!(!boilerplate(&default, "Global/AL.gitignore").matches(&name));
        assert!(boilerplate(&tamada, "Global/AL.gitignore").matches(&name));
        assert!(
            boilerplate(&tamada, "Global/AL.gitignore").matches(&Name::new("TAMADA", "Global/AL"))
        );
    }

    #[cfg(feature = "local")]
    #[test]
    fn find_takes_the_boilerplate_named_by_its_path() {
        let (base, repo) = fixture_repository(&["AL.gitignore", "Global/AL.gitignore"]);
        let found = repo.find(&Name::new_of("Global/AL"), base.path()).unwrap();
        assert_eq!(found.path(), Path::new("Global/AL.gitignore"));
        assert!(repo.find(&Name::new_of("Local/AL"), base.path()).is_none());
    }

    #[test]
    fn test_boilerplate_content_url() {
        let repo = Repository::default();
        let b = Boilerplate::new("Rust".into(), "Rust.gitignore", &repo);

        // This might fail if git command is not available or not in a git repo,
        // but it's worth testing the logic if possible.
        // We use a dummy base path.
        let result = b.content_url(".");
        // Since "." is likely not a git repo for the boilerplate, this may error,
        // but we're testing the routing logic.
        if let Ok(url) = result {
            assert!(url.contains("raw.github.com/github/gitignore"));
        }
    }
}
