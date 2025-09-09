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
//! let gixor = GixorBuilder::load("testdata/config.json").unwrap();
//! gixor.prepare().unwrap(); // clone or update all repositories, if needed.
//! let names = vec!["rust", "macos", "linux", "windows"]
//!     .iter().map(|s| Name::parse(s)).collect();
//! // dump the boilerplate of rust, macos, linux, and windows into stdout.
//! let r = gixor.dump(names, std::io::stdout());
//! ```
use std::{
    fmt::{Display, Write},
    io::BufRead,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer, Serialize};

pub mod alias;
mod git;
mod utils;

/// Represents the result of Gixor.
pub type Result<T> = std::result::Result<T, GixorError>;

/// Represents an error of Gixor.
#[derive(Debug)]
pub enum GixorError {
    Array(Vec<GixorError>),
    Alias(String),
    AliasNotFound(String),
    BoilerplateNotFound(String),
    FileNotFound(PathBuf),
    Fatal(String),
    Git(git2::Error),
    IO(std::io::Error),
    Json(serde_json::Error),
    RepositoryNotFound(String),
}

impl Display for GixorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GixorError::*;
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

/// Represents a boilerplate file.
pub struct Boilerplate<'a> {
    /// The boilerplate name. It is the stem of the boilerplate file.
    name: String,
    /// The path of the boilerplate file.
    path: PathBuf,
    /// The repository of this boilerplate.
    repo: &'a Repository,
    /// The base path of the boilerplate file.
    base_path: PathBuf,
}

impl<'a> Boilerplate<'a> {
    fn new<P: AsRef<Path>>(
        name: String,
        path: PathBuf,
        repo: &'a Repository,
        base_path: P,
    ) -> Boilerplate<'a> {
        Boilerplate {
            name,
            path,
            repo,
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub fn name(&self) -> Name {
        Name::new(self.repository_name(), self.boilerplate_name())
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
    pub fn matches(&self, name: &Name) -> bool {
        name.matches(self)
    }

    pub fn content_url(&self) -> Result<String> {
        let hash = self.repo.hash(&self.base_path)?;
        log::info!("hash: {hash:02x?}");
        let hash_string = hash.iter().fold(String::new(), |mut output, b| {
            let _ = write!(output, "{b:02X}");
            output
        });
        let url = &self.repo.url;
        let relative_path = to_relative_path(&self.path, self.base_path.join(&self.repo.name));
        if url.contains("github.com") {
            Ok(format!("https://raw.github.com/{0}/{1}/{2}/{3}", 
                self.repo.owner, self.repo.repo_name, hash_string, relative_path))
        } else if url.contains("gitlab.com") {
            Ok(format!("https://gitlab.com/{0}/{1}/-/raw/{2}/{3}",
                self.repo.owner, self.repo.repo_name, hash_string, relative_path))
        } else if url.contains("bitbucket.org") {
            Ok(format!("https://bitbucket.org/{0}/{1}/raw/{2}/{3}",
                self.repo.owner, self.repo.repo_name, hash_string, relative_path))
        } else {
            Err(GixorError::Fatal(format!(
                "{}: Unsupported repository host",
                url
            )))
        }
    }

    /// Returns the content of the boilerplate file.
    pub fn dump(&self) -> Result<String> {
        let content = dump_path(self.path.clone())?;
        Ok(format!(
            r#"### Generated by Gixor (https://github.com/tamada/gixor) ({}/{})
### {}
{}
"#,
            self.repository_name(),
            self.name,
            self.content_url()?,
            content
        ))
    }
}

fn to_relative_path(path: &Path, base_path: PathBuf) -> String {
    let relative_path = path.strip_prefix(base_path).unwrap();
    relative_path.to_string_lossy().to_string()
}

fn dump_path(path: PathBuf) -> Result<String> {
    let mut result = vec![];
    match std::fs::File::open(&path) {
        Err(e) => Err(GixorError::IO(e)),
        Ok(file) => {
            let reader = std::io::BufReader::new(file);
            for line in reader.lines() {
                result.push(line.unwrap());
            }
            Ok(result.join("\n"))
        }
    }
}

mod routine;

/// Finds the entries of `.gitignore` file in the given path.
/// The given path should be a directory containing a `.gitignore` file or a `.gitignore` file directly.
/// If the `.gitignore` file is not found, returns error.
pub fn entries<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    routine::entries(path)
}

pub fn find_target_repositories<S: AsRef<str>>(
    gixor: &Gixor,
    repository_names: Vec<S>,
) -> Result<Vec<&Repository>> {
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

    /// Returns `true` if the given boilerplate is matched with this instance.
    pub fn matches(&self, boilerplate: &Boilerplate) -> bool {
        self.boilerplate_name.to_lowercase() == boilerplate.name.to_lowercase()
            && self
                .repository_name
                .as_ref()
                .map(|s| s.to_lowercase() == boilerplate.repo.name.to_lowercase())
                .unwrap_or(true)
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
    fn repositories(&self) -> impl Iterator<Item = &Repository>;
    /// Find the repository by the name.
    fn repository<N: AsRef<str>>(&self, name: N) -> Option<&Repository>;
    /// Add the given new repository and returns the new instance of Gixor.
    fn add_repository(&mut self, repo: Repository) -> Result<()>;
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
    fn iter_aliases(&self) -> impl Iterator<Item = &alias::Alias>;
    /// Remove the alias which has the given name.
    fn remove_alias<S: AsRef<str>>(&mut self, name: S) -> Result<()>;
    /// Add the given alias.
    fn add_alias(&mut self, alias: alias::Alias) -> Result<()>;
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
                let repositories = vec![Repository::default()];
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
pub struct GixorBuilder {}

impl GixorBuilder {
    /// Load the configuration file from the default location.
    /// The default configuration is provided by [`Gixor::default`].
    pub fn load_or_default() -> Gixor {
        match dirs::config_dir() {
            Some(dir) => {
                let path = dir.join("gixor").join("config.json");
                GixorBuilder::load(path).unwrap_or_default()
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
                    repositories: vec![Repository::default()],
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
                Err(e) => Err(GixorError::Json(e)),
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
    pub fn dump(&self, names: Vec<Name>, dest: impl std::io::Write) -> Result<()> {
        match routine::find_boilerplates(self, names) {
            Err(e) => Err(e),
            Ok(boilerplates) => routine::dump_boilerplates_impl(dest, boilerplates),
        }
    }

    /// If the destination is `"-"`, the content is written to the stdout, and
    /// the `dest` is a directory, the content is written to the `${dest}/.gitignore`.
    /// Otherwise, the content is written to the file of `dest`.
    pub fn dump_to<P: AsRef<Path>>(&self, names: Vec<Name>, dest: P) -> Result<()> {
        let out = routine::open_dest(dest.as_ref())?;
        self.dump(names, out)
    }

    /// Store the configuration to the configuration path.
    pub fn store(&self) -> Result<()> {
        match std::fs::create_dir_all(self.load_from.parent().unwrap()) {
            Err(e) => Err(GixorError::IO(e)),
            Ok(_) => match std::fs::File::create(&self.load_from) {
                Err(e) => Err(GixorError::IO(e)),
                Ok(f) => match serde_json::to_writer(f, &self.config) {
                    Err(e) => Err(GixorError::Json(e)),
                    Ok(_) => Ok(()),
                },
            },
        }
    }

    /// Iterate the boilerplate paths in the configuration.
    pub fn iter(&self) -> impl Iterator<Item = Boilerplate<'_>> {
        self.config.iter()
    }

    /// Find the boilerplate by the name.
    pub fn find(&self, name: Name) -> Result<Vec<Boilerplate<'_>>> {
        self.config.find(name)
    }
}

impl AliasManager for Gixor {
    fn iter_aliases(&self) -> impl Iterator<Item = &alias::Alias> {
        self.config.iter_aliases()
    }

    fn remove_alias<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        self.config.remove_alias(name)
    }

    fn add_alias(&mut self, alias: alias::Alias) -> Result<()> {
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
    fn repository<N: AsRef<str>>(&self, name: N) -> Option<&Repository> {
        let name = name.as_ref();
        self.config
            .repositories
            .iter()
            .find(|repo| repo.name == name)
    }

    /// Iterate the repositories in the configuration.
    fn repositories(&self) -> impl Iterator<Item = &Repository> {
        self.config.repositories.iter()
    }

    /// Add the given new repository and returns the new instance of Gixor.
    fn add_repository(&mut self, repo: Repository) -> Result<()> {
        match repo.clone(&self.config.base_path) {
            Err(e) => Err(e),
            Ok(_) => {
                self.config.repositories.push(repo);
                Ok(())
            }
        }
    }

    /// Add a repository build from the given url and returns the new instance of Gixor.
    fn add_repository_of<S: AsRef<str>>(&mut self, url: S) -> Result<()> {
        let repo = Repository::new(url);
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
            Err(GixorError::Fatal(format!("{name}: repository not found")))
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
    pub(crate) repositories: Vec<Repository>,
    #[serde(flatten)]
    pub(crate) aliases: Option<alias::Aliases>,
    pub(crate) base_path: PathBuf,
}

impl Config {
    /// Find the related boilerplates by the names from all of repositories.
    /// The method matches the given name with an alias and, the boilerplate name in the repository..
    fn find(&self, name: Name) -> Result<Vec<Boilerplate<'_>>> {
        if let Some(r) = alias::extract_alias(self, &name) {
            Ok(r)
        } else {
            for repo in &self.repositories {
                if let Some(item) = repo.find(&name, &self.base_path) {
                    log::trace!("{}: found from repository {}", name, item.repository_name());
                    return Ok(vec![item]);
                }
            }
            Err(GixorError::BoilerplateNotFound(name.boilerplate_name))
        }
    }

    /// Find all related boilerplates of the given names from all of repositories.
    /// The method matches the given name with an alias and the boilerplate name in the repository.
    fn find_all(&self, names: Vec<Name>) -> Result<Vec<Boilerplate<'_>>> {
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
    fn iter(&self) -> impl Iterator<Item = Boilerplate<'_>> {
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
            utils::errs_vec_to_result(errs, ())
        }
    }
}

impl AliasManager for Config {
    fn iter_aliases(&self) -> impl Iterator<Item = &alias::Alias> {
        self.aliases.iter().flat_map(|a| a.iter_aliases())
    }

    fn remove_alias<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        self.aliases.as_mut().map_or(
            Err(GixorError::AliasNotFound(name.as_ref().to_string())),
            |aliases| aliases.remove_alias(name),
        )
    }

    fn add_alias(&mut self, alias: alias::Alias) -> Result<()> {
        let aliases = self.aliases.as_mut().unwrap();
        aliases.add_alias(alias)
    }
}

/// Represents a repository of the boilerplates.
/// The boilerplate repository is cloned into `${base_path}/${name}`
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Repository {
    /// The name of the repository. The default name is the owner name of the repository.
    pub name: String,
    /// The url of remote repository. The `url` should be formatted in the GitHub repository URL
    /// which should has owner name and repository name.
    pub url: String,
    /// The owner name of the repository.
    pub owner: String,
    /// The repository name.
    pub repo_name: String,
    /// The path of the repository.
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
    /// The name of the repository is owner name whcih is extracted from the url.
    pub fn new<S: AsRef<str>>(url: S) -> Self {
        let url = url.as_ref();
        let (owner, repo_name) = url_to_owner_and_repo_name(url);
        let path = PathBuf::from(&owner);
        Self {
            name: owner.clone(),
            url: url.to_string(),
            owner,
            repo_name,
            path,
        }
    }

    /// Creates an instance of Repository with the given name and url.
    pub fn new_with<S: AsRef<str>>(name: S, url: S) -> Self {
        let url = url.as_ref();
        let name = name.as_ref();
        let (owner, repo_name) = url_to_owner_and_repo_name(url);
        Self {
            name: name.to_string(),
            url: url.to_string(),
            repo_name,
            owner,
            path: PathBuf::from(name),
        }
    }

    fn path<P: AsRef<Path>>(&self, base_path: P) -> PathBuf {
        if self.path.is_absolute() {
            self.path.clone()
        } else {
            base_path.as_ref().join(&self.path)
        }
    }

    fn hash<P: AsRef<Path>>(&self, base_path: P) -> Result<Vec<u8>> {
        let path = base_path.as_ref().join(&self.name);
        log::trace!("try to open the git repository: {}", path.display());
        let gitrepo = match git2::Repository::open(path.clone()) {
            Ok(repo) => repo,
            Err(_) => {
                let message = format!(
                    "{} ({}): Failed to open the repository",
                    self.name.as_str(),
                    path.display()
                );
                return Err(GixorError::Git(git2::Error::from_str(message.as_str())));
            }
        };
        let head = gitrepo.head();
        match head {
            Ok(head) => match head.peel_to_commit() {
                Ok(commit) => Ok(commit.id().as_bytes().to_vec()),
                Err(e) => Err(GixorError::Git(e)),
            },
            Err(_) => Err(GixorError::Git(git2::Error::from_str(
                "Failed to get the HEAD",
            ))),
        }
    }

    /// Finds the boilerplate by the name.
    pub fn find<P: AsRef<Path>>(&self, name: &Name, base_path: P) -> Option<Boilerplate<'_>> {
        self.iter(base_path).find(|b| name.matches(b))
    }

    /// Iterates the boilerplates in the repository.
    pub fn iter<P: AsRef<Path>>(&self, base_path: P) -> impl Iterator<Item = Boilerplate<'_>> {
        let bpath = base_path.as_ref().to_path_buf();
        let path = self.path(base_path);
        ignore::WalkBuilder::new(path)
            .standard_filters(true)
            .build()
            .flatten()
            .map(|entry| entry.into_path())
            .filter(|p| is_gitignore_file(p.file_name()))
            .map(move |path| {
                Boilerplate::new(
                    path.file_stem().unwrap().to_string_lossy().to_string(),
                    path,
                    self,
                    bpath.clone(),
                )
            })
    }

    /// Prepare the repository by cloning or pulling the remote repository.
    /// If the repository already exists, do `git pull origin main`.
    /// Otherwise, execute `git clone` and store the repository into the `[self.path(base_path)]`.
    pub fn prepare<P: AsRef<Path>>(&self, base_path: P) -> Result<()> {
        // TODO: implement
        let path = self.path(&base_path);
        if path.exists() {
            let dot_git = path.join(".git");
            if dot_git.exists() {
                log::trace!("{}: repository exists", path.display());
                log::info!("Pulling {} to {}", self.url, path.display());
                match git::pull(&path, "origin", "main") {
                    Ok(_) => Ok(()),
                    Err(e) => Err(GixorError::Git(e)),
                }
            } else {
                log::info!("Cloning {} to {}", self.url, path.display());
                git::clone(&self.url, &path)
            }
        } else {
            self.clone(base_path)
        }
    }

    /// Clone the repository into the `[self.path(base_path)]`.
    /// If the repository already exists, do nothing and returns `Ok(())`.
    fn clone<P: AsRef<Path>>(&self, base_path: P) -> Result<()> {
        let path = self.path(base_path);
        if path.exists() {
            let dot_git = path.join(".git");
            if dot_git.exists() {
                log::info!("{}: repository exists", path.display());
                Ok(())
            } else {
                log::info!("Cloning {} to {}", self.url, path.display());
                git::clone(&self.url, &path)
            }
        } else {
            log::info!("Cloning {} to {}", self.url, path.display());
            git::clone(&self.url, &path)
        }
    }
}

fn is_gitignore_file(name: Option<&std::ffi::OsStr>) -> bool {
    if let Some(name) = name.unwrap().to_str() {
        name.ends_with(".gitignore")
    } else {
        false
    }
}

fn url_to_owner_and_repo_name(url: &str) -> (String, String) {
    let items = url.split('/').collect::<Vec<_>>();
    match (items.get(items.len() - 2), items.last()) {
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

fn remove_repo_dir<P: AsRef<Path>>(base_path: P, repo: Repository) -> Result<()> {
    let path = base_path.as_ref().join(repo.name);
    match std::fs::remove_dir_all(&path) {
        Err(e) => Err(GixorError::IO(e)),
        Ok(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use serde::de::Error;

    use super::*;

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
    fn parse_gixor() {
        match GixorBuilder::load(PathBuf::from("../testdata/config.json")) {
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
    fn test_repo() {
        let repo = Repository::new_with("tamada", "https://github.com/tamada/gitignore.git");
        assert_eq!(repo.name, "tamada");
        assert_eq!(repo.url, "https://github.com/tamada/gitignore.git");

        let base_path = PathBuf::from("../testdata/boilerplates");
        if let Err(e) = repo.prepare(&base_path) {
            panic!("Failed to prepare the repository: {e}");
        }
        let boilerplates = repo.iter(&base_path).collect::<Vec<_>>();
        assert_eq!(boilerplates.len(), 1);

        if let Some(b) = repo.find(&Name::new_of("devcontainer"), &base_path) {
            assert_eq!(
                b.path,
                PathBuf::from("../testdata/boilerplates/tamada/devcontainer.gitignore")
            );
        } else {
            panic!("Failed to find the devcontainer.gitignore");
        }
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            GixorError::Json(serde_json::Error::custom("hoge")).to_string(),
            "JSON error: hoge"
        );
        assert_eq!(
            GixorError::IO(std::io::Error::new(std::io::ErrorKind::NotFound, "hoge")).to_string(),
            "IO error: hoge"
        );
        assert_eq!(
            GixorError::BoilerplateNotFound("name".to_string()).to_string(),
            "name: boilerplate not found"
        );
        assert_eq!(
            GixorError::Git(git2::Error::from_str("hoge")).to_string(),
            "Git error: hoge"
        );
        assert_eq!(
            GixorError::AliasNotFound("hoge".into()).to_string(),
            "hoge: alias not found"
        );
        assert_eq!(
            GixorError::FileNotFound("hoge".into()).to_string(),
            "hoge: file not found"
        );
        assert_eq!(
            GixorError::RepositoryNotFound("hoge".into()).to_string(),
            "hoge: repository not found"
        );
        assert_eq!(
            GixorError::Fatal("message".to_string()).to_string(),
            "Fatal error: message"
        );
        assert_eq!(
            GixorError::Array(vec![
                GixorError::Fatal("hoge1".to_string()),
                GixorError::Fatal("hoge2".to_string())
            ])
            .to_string(),
            "Fatal error: hoge1Fatal error: hoge2"
        );
        assert_eq!(
            GixorError::Alias("hoge: alias not found".to_string()).to_string(),
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
