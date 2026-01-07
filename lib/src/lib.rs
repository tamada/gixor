//! Gixor is a tool to manage the boilerplate files (`.gitignore`).
//! This is alternative tool of [gibo](https://github.com/simonwhitaker/gibo) written in Rust.
//!
//! Also, this library provides an API of the Gitignore boilerplate management.
//! The main structure of this library is [`Gixor`].
//!
//! # Example of Dump the boilerplate
//!
//! ```rust
//! use gixor::{Gixor, GixorBuilder, Name, Result};
//!
//! // load configuration file and build Gixor object.
//! let gixor = GixorBuilder::load("testdata/config.json").unwrap();
//! gixor.prepare(true).unwrap(); // clone or update all repositories, if needed.
//! // create vec of Name instance.
//! let names = Name::parse_all(vec!["rust", "macos", "linux", "windows"])
//! // dump the boilerplate of rust, macos, linux, and windows into stdout.
//! let r = gixor.dump(names, std::io::stdout());
//! ```
//!
//! # Features
//!
//! [`Gixor`] provides the following features for operating Git repositories.:
//! - `uselibgit`: use [`git2`](https://docs.rs/git2/latest/git2/) crate which uses libgit2 C library.
//! - `usegix`: use [`gix`](https://docs.rs/gix/latest/gix/) crate which is a pure Rust implementation of Git.
//! - default: use `git` command via [`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html).
//!
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer, Serialize};

pub mod aliases;
pub mod gitbridge;
pub mod repos;

/// Represents the result of Gixor.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents an error of Gixor.
#[derive(Debug)]
pub enum Error {
    Array(Vec<Error>),
    Alias(String),
    AliasNotFound(String),
    BoilerplateNotFound(String),
    FileNotFound(PathBuf),
    Fatal(String),
    Git(String),
    IO(std::io::Error),
    Json(serde_json::Error),
    RepositoryNotFound(String),
}

impl Error {
    pub fn to_err<T>(item: T, errs: Vec<Error>) -> Result<T> {
        if errs.is_empty() {
            Ok(item)
        } else if errs.len() == 1 {
            Err(errs.into_iter().next().unwrap())
        } else {
            Err(Error::Array(errs))
        }
    }

    /// Convert `Vec<Result<T>>` to `Result<Vec<T>>`
    /// If `Vec<Result<T>>` has the multiple errors,
    /// `Result<Vec<T>>` returns `Err(GixorError::Array(Vec<GixorError>))`.
    pub fn vec_result_to_result_vec<T>(vec: Vec<Result<T>>) -> Result<Vec<T>> {
        let mut ok_items = vec![];
        let mut errs = vec![];
        for r in vec {
            match r {
                Ok(item) => ok_items.push(item),
                Err(e) => errs.push(e),
            }
        }
        Error::to_err(ok_items, errs)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Array(errs) => {
                let result = errs.iter().map(|e| e.fmt(f)).collect::<Vec<_>>();
                if result.iter().any(|r| r.is_err()) {
                    Err(std::fmt::Error)
                } else {
                    Ok(())
                }
            }
            Alias(msg) => write!(f, "{msg}"),
            AliasNotFound(name) => write!(f, "{name}: alias not found"),
            BoilerplateNotFound(name) => write!(f, "{name}: boilerplate not found"),
            FileNotFound(path) => write!(f, "{}: file not found", path.display()),
            Git(e) => write!(f, "Git error: {e}"),
            IO(e) => write!(f, "IO error: {e}"),
            Json(e) => write!(f, "JSON error: {e}"),
            Fatal(msg) => write!(f, "Fatal error: {msg}"),
            RepositoryNotFound(name) => write!(f, "{name}: repository not found"),
        }
    }
}

mod routine;

/// Finds the entries of `.gitignore` file in the given path.
/// The given path should be a directory containing a `.gitignore` file or a `.gitignore` file directly.
/// If the `.gitignore` file is not found, returns error.
pub fn entries<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    log::info!("Find current entries from {}", path.as_ref().display());
    routine::entries(path)
}

pub fn find_target_repositories<S: AsRef<str>>(
    gixor: &Gixor,
    repository_names: Vec<S>,
) -> Result<Vec<&repos::Repository>> {
    log::info!("find_target_repositories: repository_names={:?}", repository_names.iter().map(|s| s.as_ref()).collect::<Vec<_>>());
    routine::find_target_repositories(gixor, repository_names)
}

/// The name of the boilerplate which contains the repository name and the boilerplate name.
/// The repository name is [`Repository::name`].
/// The boilerplate name is the file stem of the boilerplate (gitignore) file.
#[derive(Debug, Clone)]
pub struct Name {
    /// The repository name for of the boilerplate. If `None`, the repository name do not care.
    pub repository_name: Option<String>,
    /// The boilerplate name.
    pub boilerplate_name: String,
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Name::parse(s))
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.repository_name {
            Some(repo) => write!(f, "{}/{}", repo, self.boilerplate_name),
            None => write!(f, "{}", self.boilerplate_name),
        }
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Name::parse(s)
    }
}

/// Represents a boilerplate name for finding a boilerplate.
impl Name {
    /// Create a new `Name` instance with boilerplate name.
    /// The repository name is `None` (`None` means don't care).
    fn new_of<S: AsRef<str>>(boilerplate_name: S) -> Self {
        Self {
            repository_name: None,
            boilerplate_name: boilerplate_name.as_ref().to_string(),
        }
    }

    /// Create a new `Name` instance with repository name and boilerplate name.
    pub fn new<S: AsRef<str>>(repository_name: S, boilerplate_name: S) -> Self {
        let boilerplate_name = boilerplate_name.as_ref().to_string();
        Self {
            repository_name: Some(repository_name.as_ref().to_string()),
            boilerplate_name,
        }
    }

    /// Create a new `Name` instance with the given name.
    /// The given name should format `<repository_name>/<boilerplate_name>`.
    /// If the given string do not contain `/`, the repository name is `None`.
    pub fn parse<S: AsRef<str>>(name: S) -> Self {
        let name = name.as_ref();
        let items = name.split('/').collect::<Vec<_>>();
        if items.len() >= 2 {
            Self::new(items[0], items[1])
        } else {
            Self::new_of(name)
        }
    }

    /// Create a vec of `Name` instance from the given string vec.
    /// The this method gives each name to [Name::parse] method, and collect them.
    pub fn parse_all<S: AsRef<str>>(names: Vec<S>) -> Vec<Self> {
        names.iter().map(Name::parse).collect()
    }

    /// Returns `true` if the given boilerplate is matched with this instance.
    pub fn matches(&self, boilerplate: &repos::Boilerplate) -> bool {
        boilerplate.matches(self)
    }
}

/// Represents the main structure of Gixor.
pub struct Gixor {
    config: Config,
    load_from: PathBuf,
}

/// Provides the functions for management of the boilerplate repositories.
pub trait RepositoryManager {
    /// Returns the length of the repositories in the container.
    fn len(&self) -> usize;
    /// Returns `true` if the repositories in the container is empty.
    fn is_empty(&self) -> bool;
    /// Iterate the repositories in the container.
    fn repositories(&self) -> impl Iterator<Item = &repos::Repository>;
    /// Find the repository by the name.
    fn repository<N: AsRef<str>>(&self, name: N) -> Option<&repos::Repository>;
    /// Add the given new repository and returns the new instance of Gixor.
    fn add_repository(&mut self, repo: repos::Repository) -> Result<()>;
    /// Add a repository build from the given url and returns the new instance of Gixor.
    fn add_repository_of<S: AsRef<str>>(&mut self, url: S) -> Result<()>;
    /// Remove the repository which has the given name, and returns the new instance of Gixor.
    fn remove_repository_with<S: AsRef<str>>(&mut self, name: S, keep_repo_dir: bool)
        -> Result<()>;
    /// Remove the repository which has the given name, and returns the new instance of Gixor.
    fn remove_repository<S: AsRef<str>>(&mut self, name: S) -> Result<()>;
}

/// Provides the functions for management of the aliases.
pub trait AliasManager {
    /// Iterate the aliases in the configuration.
    fn iter_aliases(&self) -> impl Iterator<Item = &aliases::Alias>;
    /// Remove the alias which has the given name.
    fn remove_alias<S: AsRef<str>>(&mut self, name: S) -> Result<()>;
    /// Add the given alias.
    fn add_alias(&mut self, alias: aliases::Alias) -> Result<()>;
}

impl Default for Gixor {
    /// Create a default instance of Gixor.
    /// The default configuration is as follows:
    /// - The base path is as follows.
    ///     - Linux: `$XDG_CONFIG_HOME/gixor/config.json` or `$HOME/.config/gixor/config.json`
    ///     - macOS: `$HOME/Library/Application Support/gixor/config.json`
    ///     - Windows: `{FOLDERID_RoamingAppData}\gixor\config.json`
    /// - The default repository is [`Repository::default`].
    /// - The default configuration file is `${XDG_CONFIG_HOME}/gixor/config.json`.
    ///
    /// The default location is as follows.
    fn default() -> Self {
        match dirs::config_dir() {
            Some(dir) => {
                let repositories = vec![repos::Repository::default()];
                let config = Config {
                    repositories,
                    base_path: dir.join("gixor").join("boilerplates"),
                    aliases: None,
                };
                Self {
                    config,
                    load_from: dir.join("gixor").join("config.json"),
                }
            }
            None => panic!("Failed to get the config directory"),
        }
    }
}

/// The builder of [`Gixor`].
pub struct GixorFactory {}

impl GixorFactory {
    /// Load the configuration file from the default location.
    /// The default configuration is provided by [`Gixor::default`].
    pub fn load_or_default() -> Gixor {
        match dirs::config_dir() {
            Some(dir) => {
                let path = dir.join("gixor").join("config.json");
                GixorFactory::load(path).unwrap_or_default()
            }
            None => panic!("Failed to get the config directory"),
        }
    }

    /// Parse the configuration file from the given path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Gixor> {
        let path = path.as_ref();
        match std::fs::File::open(path) {
            Err(_) => Ok(Gixor::new(
                Config {
                    repositories: vec![repos::Repository::default()],
                    base_path: path.parent().unwrap().join("boilerplates"),
                    aliases: None,
                },
                path.to_path_buf(),
            )),
            Ok(f) => match serde_json::from_reader(f) {
                Ok(config) => Ok(Gixor::new(
                    update_base_path(config, path),
                    path.to_path_buf(),
                )),
                Err(e) => Err(Error::Json(e)),
            },
        }
    }
}

impl Gixor {
    fn new(config: Config, load_from: PathBuf) -> Self {
        Gixor { config, load_from }
    }
    /// Returns the base path of this configuration.
    pub fn base_path(&self) -> &Path {
        &self.config.base_path
    }

    /// Prepare the repositories in the local environment by cloning or updating them.
    pub fn prepare(&self, no_network: bool) -> Result<()> {
        self.config.prepare(no_network)
    }

    /// Write the the content of boilerplate corresponding the given names to the destination.
    pub fn dump(
        &self,
        names: Vec<Name>,
        dest: impl std::io::Write,
        clear_flag: bool,
    ) -> Result<()> {
        match routine::find_boilerplates(self, names) {
            Err(e) => Err(e),
            Ok(boilerplates) => routine::dump_boilerplates_impl(dest, boilerplates, clear_flag, self.base_path()),
        }
    }

    /// If the destination is `"-"`, the content is written to the stdout, and
    /// the `dest` is a directory, the content is written to the `${dest}/.gitignore`.
    /// Otherwise, the content is written to the file of `dest`.
    pub fn dump_to<P: AsRef<Path>>(
        &self,
        names: Vec<Name>,
        dest: P,
        clear_flag: bool,
    ) -> Result<()> {
        let p = dest.as_ref();
        log::info!(
            "dump {} entries into {} with clear_flag: {clear_flag}.",
            names.len(),
            p.display()
        );
        let out = routine::open_dest(p)?;
        self.dump(names, out, clear_flag)
    }

    /// Store the configuration to the configuration path.
    pub fn store(&self) -> Result<()> {
        match std::fs::create_dir_all(self.load_from.parent().unwrap()) {
            Err(e) => Err(Error::IO(e)),
            Ok(_) => match std::fs::File::create(&self.load_from) {
                Err(e) => Err(Error::IO(e)),
                Ok(f) => match serde_json::to_writer(f, &self.config) {
                    Err(e) => Err(Error::Json(e)),
                    Ok(_) => Ok(()),
                },
            },
        }
    }

    /// Iterate the boilerplate paths in the configuration.
    pub fn iter(&self) -> impl Iterator<Item = repos::Boilerplate<'_>> {
        self.config.iter()
    }

    /// Find the boilerplate by the name.
    pub fn find(&self, name: Name) -> Result<Vec<repos::Boilerplate<'_>>> {
        self.config.find(name)
    }
}

impl AliasManager for Gixor {
    fn iter_aliases(&self) -> impl Iterator<Item = &aliases::Alias> {
        self.config.iter_aliases()
    }

    fn remove_alias<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        self.config.remove_alias(name)
    }

    fn add_alias(&mut self, alias: aliases::Alias) -> Result<()> {
        self.config.add_alias(alias)
    }
}

impl RepositoryManager for Gixor {
    /// Find the repository by the name.
    /// Returns the length of the repositories in the configuration.
    fn len(&self) -> usize {
        self.config.repositories.len()
    }

    /// Returns `true` if the repositories in the configuration is empty.
    fn is_empty(&self) -> bool {
        self.config.repositories.is_empty()
    }

    /// Find the repository by the name.
    fn repository<N: AsRef<str>>(&self, name: N) -> Option<&repos::Repository> {
        let name = name.as_ref();
        self.config
            .repositories
            .iter()
            .find(|repo| repo.name == name)
    }

    /// Iterate the repositories in the configuration.
    fn repositories(&self) -> impl Iterator<Item = &repos::Repository> {
        self.config.repositories.iter()
    }

    /// Add the given new repository and returns the new instance of Gixor.
    fn add_repository(&mut self, repo: repos::Repository) -> Result<()> {
        match repo.clone_repo_to(&self.config.base_path) {
            Err(e) => Err(e),
            Ok(_) => {
                self.config.repositories.push(repo);
                Ok(())
            }
        }
    }

    /// Add a repository build from the given url and returns the new instance of Gixor.
    fn add_repository_of<S: AsRef<str>>(&mut self, url: S) -> Result<()> {
        let repo = repos::Repository::new(url);
        self.add_repository(repo)
    }

    /// Remove the repository which has the given name, and returns the new instance of Gixor.
    /// If `keep_repo_dir` is `true`, the directory of the removed repository will be remained.
    fn remove_repository_with<S: AsRef<str>>(
        &mut self,
        name: S,
        keep_repo_dir: bool,
    ) -> Result<()> {
        let name = name.as_ref();
        let index = self
            .config
            .repositories
            .iter()
            .position(|repo| repo.name == name);
        if let Some(index) = index {
            let repo = self.config.repositories.remove(index);
            if !keep_repo_dir {
                remove_repo_dir(&self.config.base_path, repo)?;
            }
            Ok(())
        } else {
            Err(Error::Fatal(format!("{name}: repository not found")))
        }
    }

    /// Remove the repository which has the given name, and returns the new instance of Gixor.
    /// The directory of the removed repository will be deleted.
    fn remove_repository<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        self.remove_repository_with(name, false)
    }
}

fn update_base_path(config: Config, path: &Path) -> Config {
    let parent = path.parent().unwrap();
    let base_path = config.base_path.clone();
    let new_base_path = if base_path.is_absolute() || base_path.starts_with(".") {
        base_path
    } else {
        parent.join(base_path)
    };
    Config {
        base_path: new_base_path,
        repositories: config.repositories,
        aliases: config.aliases,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Config {
    pub(crate) repositories: Vec<repos::Repository>,
    #[serde(flatten)]
    pub(crate) aliases: Option<aliases::Aliases>,
    pub(crate) base_path: PathBuf,
}

impl Config {
    /// Find the related boilerplates by the names from all of repositories.
    /// The method matches the given name with an alias and, the boilerplate name in the repository..
    fn find(&self, name: Name) -> Result<Vec<repos::Boilerplate<'_>>> {
        if let Some(r) = aliases::extract_alias(self, &name) {
            Ok(r)
        } else {
            for repo in &self.repositories {
                if let Some(item) = repo.find(&name, &self.base_path) {
                    log::trace!("{}: found from repository {}", name, item.repository_name());
                    return Ok(vec![item]);
                }
            }
            Err(Error::BoilerplateNotFound(name.boilerplate_name))
        }
    }

    /// Find all related boilerplates of the given names from all of repositories.
    /// The method matches the given name with an alias and the boilerplate name in the repository.
    fn find_all(&self, names: Vec<Name>) -> Result<Vec<repos::Boilerplate<'_>>> {
        let r = names
            .into_iter()
            .map(|name| self.find(name))
            .collect::<Result<Vec<_>>>();
        match r {
            Ok(v) => Ok(v.into_iter().flatten().collect::<Vec<_>>()),
            Err(e) => Err(e),
        }
    }

    /// Iterate the boilerplates from all repositories.
    fn iter(&self) -> impl Iterator<Item = repos::Boilerplate<'_>> {
        self.repositories
            .iter()
            .flat_map(move |repo| repo.iter(&self.base_path))
    }

    /// Prepare the repositories in the local environment by cloning or updating them.
    fn prepare(&self, no_network: bool) -> Result<()> {
        let mut errs = vec![];
        if no_network {
            log::info!("Network access is disabled.");
            Ok(())
        } else {
            self.repositories.iter().for_each(|repo| {
                if let Err(e) = repo.prepare(&self.base_path) {
                    errs.push(e);
                }
            });
            Error::to_err((), errs)
        }
    }
}

impl AliasManager for Config {
    fn iter_aliases(&self) -> impl Iterator<Item = &aliases::Alias> {
        self.aliases.iter().flat_map(|a| a.iter_aliases())
    }

    fn remove_alias<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        self.aliases.as_mut().map_or(
            Err(Error::AliasNotFound(name.as_ref().to_string())),
            |aliases| aliases.remove_alias(name),
        )
    }

    fn add_alias(&mut self, alias: aliases::Alias) -> Result<()> {
        let aliases = self.aliases.as_mut().unwrap();
        aliases.add_alias(alias)
    }
}

fn remove_repo_dir<P: AsRef<Path>>(base_path: P, repo: repos::Repository) -> Result<()> {
    let path = base_path.as_ref().join(repo.name);
    match std::fs::remove_dir_all(&path) {
        Err(e) => Err(Error::IO(e)),
        Ok(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vec_result_to_result_vec() {
        let value = vec![Ok(1), Ok(2), Ok(3)];
        let result = Error::vec_result_to_result_vec(value).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn parse_gixor() {
        match GixorFactory::load(PathBuf::from("../testdata/config.json")) {
            Err(e) => panic!("Failed to parse the config file: {e}"),
            Ok(gixor) => {
                assert_eq!(
                    gixor.config.base_path,
                    PathBuf::from("../testdata/boilerplates")
                );
                assert_eq!(gixor.config.repositories.len(), 3);
            }
        }
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            Error::Json(serde::de::Error::custom("hoge")).to_string(),
            "JSON error: hoge"
        );
        assert_eq!(
            Error::IO(std::io::Error::new(std::io::ErrorKind::NotFound, "hoge")).to_string(),
            "IO error: hoge"
        );
        assert_eq!(
            Error::BoilerplateNotFound("name".to_string()).to_string(),
            "name: boilerplate not found"
        );
        assert_eq!(
            Error::Git("hoge".into()).to_string(),
            "Git error: hoge"
        );
        assert_eq!(
            Error::AliasNotFound("hoge".into()).to_string(),
            "hoge: alias not found"
        );
        assert_eq!(
            Error::FileNotFound("hoge".into()).to_string(),
            "hoge: file not found"
        );
        assert_eq!(
            Error::RepositoryNotFound("hoge".into()).to_string(),
            "hoge: repository not found"
        );
        assert_eq!(
            Error::Fatal("message".to_string()).to_string(),
            "Fatal error: message"
        );
        assert_eq!(
            Error::Array(vec![
                Error::Fatal("hoge1".to_string()),
                Error::Fatal("hoge2".to_string())
            ])
            .to_string(),
            "Fatal error: hoge1Fatal error: hoge2"
        );
        assert_eq!(
            Error::Alias("hoge: alias not found".to_string()).to_string(),
            "hoge: alias not found"
        )
    }

    #[test]
    fn test_target_name() {
        let target = Name::new("tamada", "devcontainer");
        assert_eq!(target.repository_name, Some("tamada".to_string()));
        assert_eq!(target.boilerplate_name, "devcontainer");

        let target = Name::parse("tamada/devcontainer");
        assert_eq!(target.repository_name, Some("tamada".to_string()));
        assert_eq!(target.boilerplate_name, "devcontainer");

        let target = Name::parse("devcontainer");
        assert_eq!(target.repository_name, None);
        assert_eq!(target.boilerplate_name, "devcontainer");
    }

    #[test]
    fn test_name_serialize_deserialize() {
        let name: Name = serde_json::from_str("\"os-list\"").unwrap();
        assert_eq!(name.repository_name, None);
        assert_eq!(name.boilerplate_name, "os-list");

        let str = serde_json::to_string(&name).unwrap();
        assert_eq!(str, "\"os-list\"");

        let name: Name = serde_json::from_str("\"alias/os-list\"").unwrap();
        assert_eq!(name.repository_name, Some("alias".to_string()));
        assert_eq!(name.boilerplate_name, "os-list");

        let str = serde_json::to_string(&name).unwrap();
        assert_eq!(str, "\"alias/os-list\"");
    }
}
