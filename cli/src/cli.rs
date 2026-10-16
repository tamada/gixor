use std::path::PathBuf;

use clap::{Parser, Subcommand};
use gixor::{Gixor, Name};

/// The build information `build.rs` writes out, shown by `--version`.
const BUILD_LONG_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/BUILD_LONG_VERSION.txt"));

#[derive(Parser, Debug)]
#[command(name = "gixor", author, version, long_version = BUILD_LONG_VERSION)]
#[command(about, arg_required_else_help = true)]
pub(crate) struct CliOpts {
    #[clap(subcommand)]
    pub(crate) subcmd: GixorCommand,

    #[arg(short, long, help = "Specify the log level", default_value = "warn")]
    pub(crate) log: crate::LogLevel,

    #[arg(
        long = "no-network",
        help = "Disable network access",
        default_value_t = false
    )]
    pub(crate) no_network: bool,

    #[arg(
        short,
        long,
        value_name = "CONFIG_JSON",
        help = "Specify the configuration file"
    )]
    pub(crate) config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum GixorCommand {
    #[command(
        name = "alias",
        about = "Manage the aliases. If no command is given, list the aliases."
    )]
    Alias(AliasOpts),
    #[command(name = "dump", about = "Dump the boilerplates")]
    Dump(DumpOpts),
    #[command(
        name = "entries", aliases = ["entry"],
        about = "List the current entries in the .gitignore file"
    )]
    Entries(EntriesOpts),
    #[command(name = "init", about = "Initialize the Gixor", hide = true)]
    Init,
    #[command(name = "list", alias = "ls", about = "List available boilerplates")]
    List(ListOpts),
    #[command(name = "root", about = "Show the root directory of the boilerplates")]
    Root(RootOpts),
    #[command(
        name = "search",
        alias = "find",
        about = "Search the boilerplates from the query"
    )]
    Search(SearchOpts),
    #[command(
        name = "update",
        about = "Update the gitignore boilerplate repositories (alias of `repository update`)"
    )]
    Update,
    #[command(
        name = "repository",
        alias = "repo",
        about = "Manage the gitignore boilerplate repositories"
    )]
    #[clap(subcommand)]
    Repository(RepositoryOpts),

    #[cfg(debug_assertions)]
    #[command(
        name = "generate-completion-files",
        about = "Generate the completion files"
    )]
    CompletionFiles(CompleteOpts),
}

#[derive(Parser, Debug)]
pub(crate) struct AliasOpts {
    #[clap(subcommand)]
    pub(crate) cmd: Option<AliasCmd>,
}

#[derive(Parser, Debug)]
pub(crate) enum AliasCmd {
    #[command(name = "add", aliases = ["append", "register"], about = "Add a new alias")]
    Add(AliasAddOpts),

    #[command(name = "remove", aliases = ["delete"], about = "Remove an existing alias")]
    Remove(AliasRemoveOpts),

    #[command(name = "list", aliases = ["ls"], about = "List all aliases")]
    List,
}

#[derive(Parser, Debug)]
pub(crate) struct AliasAddOpts {
    #[clap(
        short,
        long,
        default_value = "",
        help = "Specify the alias description for registration."
    )]
    pub(crate) description: String,

    #[clap(index = 1, value_name = "NAME", help = "Specify the alias name")]
    pub(crate) name: String,

    #[clap(
        index = 2,
        value_name = "BOILERPLATE_NAMES...",
        help = "Specify the boilerplate names for the alias"
    )]
    pub(crate) boilerplates: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct AliasRemoveOpts {
    #[clap(
        index = 1,
        value_name = "NAME",
        help = "Specify the alias name for removal"
    )]
    pub(crate) args: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum RepositoryOpts {
    #[command(name = "add", about = "Add a new gitignore boilerplate repository")]
    Add(RepoAddOpts),
    #[command(
        name = "list",
        about = "List the current gitignore boilerplate repositories"
    )]
    List,
    #[command(name = "remove", about = "Remove a gitignore boilerplate repository")]
    Remove(RepoRemoveOpts),
    #[command(
        name = "update",
        about = "Run `git update` for updating a gitignore boilerplate repository"
    )]
    Update,
}

#[derive(Parser, Debug)]
pub(crate) struct RepoAddOpts {
    #[clap(
        short,
        long,
        value_name = "NAME",
        help = "Specify the name of the gitignore boilerplate repository"
    )]
    pub(crate) name: Option<String>,

    #[clap(
        value_name = "URL|NAME",
        help = r#"Specify the URL or NAME of the gitignore boilerplate repository.
The NAME shows the owner name of the repository, e.g., "github" means "https://github.com/github/gitignore""#
    )]
    pub(crate) url: String,
}

#[derive(Parser, Debug)]
pub(crate) struct RepoRemoveOpts {
    #[clap(
        short,
        long,
        default_value_t = false,
        help = "Do not remove the directory of corresponding repository"
    )]
    pub(crate) keep_dir: bool,

    #[clap(
        value_name = "NAME",
        help = "Specify the NAME of the gitignore boilerplate repository"
    )]
    pub(crate) name: String,
}

#[derive(Parser, Debug)]
pub(crate) struct DumpOpts {
    #[clap(
        short,
        long,
        value_name = "DEST",
        default_value = ".gitignore",
        help = "Specify the destination directory. \"-\" means stdout."
    )]
    pub(crate) dest: String,

    #[clap(
        long,
        help = "Drop the entries currently listed in the gitignore.",
        default_value_t = false
    )]
    pub(crate) no_append: bool,

    #[clap(
        long,
        help = "Drop the prologue, the part of the gitignore before the first boilerplate.",
        default_value_t = false
    )]
    pub(crate) clear_prologue: bool,

    #[clap(
        short,
        long,
        help = "Start from scratch, dropping both the prologue and the current entries.",
        default_value_t = false
    )]
    pub(crate) clear: bool,

    #[clap(
        short = 'n',
        long,
        help = "Print the result to stdout and leave the gitignore untouched.",
        default_value_t = false
    )]
    pub(crate) dry_run: bool,

    /// Kept so that existing scripts keep working. Appending is the default now.
    #[clap(short, long, hide = true, default_value_t = false)]
    pub(crate) append: bool,

    #[clap(value_name = "NAMES...", help = "The boilerplate names to dump.")]
    pub(crate) names: Vec<String>,
}

impl DumpOpts {
    /// Returns true if the prologue of the destination should be dropped.
    pub fn should_clear_prologue(&self) -> bool {
        self.clear || self.clear_prologue
    }

    /// Returns true if the entries already listed in the destination should be dropped.
    fn drop_current_entries(&self) -> bool {
        self.clear || self.no_append
    }

    /// Returns the target names for dumping.
    /// Unless the current entries are dropped, they are read from the destination first,
    /// then the given names are added and the `-NAME` ones removed.
    /// Finally, convert `String` to `Name` by `Name::parse` and return it.
    pub fn names(&self, gixor: &Gixor) -> gixor::Result<Vec<Name>> {
        let current = self.resolvable_current_list(gixor)?;
        Ok(self.names_with(current, |name| boilerplates_named(gixor, name)))
    }

    /// Merges the given names into `current` and parses the result.
    ///
    /// The names are compared by the boilerplates `resolve` gives for them rather than as
    /// strings, so that a name drops what the same name would add: `-macOS` drops the entry
    /// `default/Global/macOS`, `-AL` drops `default/AL` but not `default/Global/AL`, and adding
    /// `Go` again does not list it twice. A name resolving to nothing is still added, to be
    /// reported when it is resolved strictly; dropping it only warns.
    ///
    /// Split out from [`DumpOpts::names`] so the merging can be exercised without repositories.
    fn names_with<F>(&self, current: Vec<String>, resolve: F) -> Vec<Name>
    where
        F: Fn(&str) -> Vec<String>,
    {
        let mut entries = current
            .into_iter()
            .map(|name| {
                let boilerplates = resolve(&name);
                (name, boilerplates)
            })
            .collect::<Vec<_>>();
        for name in &self.names {
            if let Some(trunk) = name.strip_prefix('-') {
                let dropped = resolve(trunk);
                let before = entries.len();
                entries.retain(|(_, bs)| bs.iter().all(|b| !dropped.contains(b)));
                if entries.len() == before {
                    let listed = entries.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>();
                    warn_nothing_dropped(trunk, !dropped.is_empty(), &listed);
                }
            } else {
                let added = resolve(name);
                let listed = !added.is_empty()
                    && added
                        .iter()
                        .all(|b| entries.iter().any(|(_, bs)| bs.contains(b)));
                if !listed {
                    entries.push((name.clone(), added));
                }
            }
        }
        let v = entries
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();
        log::debug!("parse dumping targets: {}", v.join(", "));
        Name::parse_all(v)
    }

    /// Drops the current entries that no longer resolve to a boilerplate.
    ///
    /// These names come from the destination file rather than from the command line, so a
    /// boilerplate renamed or removed upstream is not the user's mistake and must not block
    /// the update. Names given on the command line are still resolved strictly, later on.
    fn resolvable_current_list(&self, gixor: &Gixor) -> gixor::Result<Vec<String>> {
        let current = self.current_list_if_append()?;
        Ok(current
            .into_iter()
            .filter(|name| match gixor.find(Name::parse(name)) {
                Ok(_) => true,
                Err(e) => {
                    log::warn!("{name}: dropped from the gitignore ({e})");
                    false
                }
            })
            .collect())
    }

    fn current_list_if_append(&self) -> gixor::Result<Vec<String>> {
        if self.drop_current_entries() {
            return Ok(vec![]);
        }
        let d = if self.dest == "-" {
            String::from(".gitignore")
        } else {
            self.dest.clone()
        };
        match gixor::entries(d) {
            // There is nothing to carry over before the gitignore exists, and creating one is
            // the ordinary case now that appending is the default.
            Err(gixor::Error::FileNotFound(_)) => Ok(vec![]),
            r => r,
        }
    }
}

/// Tells why `-name` dropped nothing, with the current entries it may have meant.
///
/// `name` either names no boilerplate at all, typically a typo, or names one the `.gitignore` does
/// not list, as `AL` does when only `Global/AL` is listed: `AL` takes `AL.gitignore`, the
/// shallowest of the boilerplates sharing the name.
fn warn_nothing_dropped(name: &str, resolved: bool, entries: &[&str]) {
    let reason = if resolved {
        "the .gitignore does not list it"
    } else {
        "no boilerplate has the name"
    };
    let candidates = suggestions(name, entries);
    if candidates.is_empty() {
        log::warn!("{name}: nothing dropped, {reason}");
    } else {
        log::warn!(
            "{name}: nothing dropped, {reason}; did you mean {}?",
            candidates.join(", ")
        );
    }
}

/// The current entries `name` may have meant: those whose file stem is `name` ignoring case, then
/// those whose file stem is a small edit away from it.
fn suggestions(name: &str, entries: &[&str]) -> Vec<String> {
    let wanted = file_stem(name).to_lowercase();
    let limit = if wanted.chars().count() >= 5 { 2 } else { 1 };
    let mut same = vec![];
    let mut similar = vec![];
    for entry in entries {
        let stem = file_stem(entry).to_lowercase();
        if stem == wanted {
            same.push(entry.to_string());
        } else if edit_distance(&stem, &wanted) <= limit {
            similar.push(entry.to_string());
        }
    }
    same.extend(similar);
    same
}

/// The last part of a name such as `default/Global/AL`, i.e. `AL`.
fn file_stem(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

/// The number of insertions, deletions, substitutions and transpositions of adjacent characters
/// turning `a` into `b`, so that a swap such as `Rsut` for `Rust` counts once.
fn edit_distance(a: &str, b: &str) -> usize {
    let a = a.chars().collect::<Vec<_>>();
    let b = b.chars().collect::<Vec<_>>();
    let mut d = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}

/// The boilerplates `name` stands for, each as `<repository>/<path>` in lower case, such as
/// `default/global/macos`; none when it names nothing.
fn boilerplates_named(gixor: &Gixor, name: &str) -> Vec<String> {
    gixor
        .find(Name::parse(name))
        .map(|bs| {
            bs.iter()
                .map(|b| format!("{}/{}", b.repository_name(), b.path_name()).to_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Parser, Debug)]
pub(crate) struct EntriesOpts {
    #[clap(
        short,
        long,
        help = "Specify the directory located the .gitignore file",
        default_value = "."
    )]
    pub(crate) dir: PathBuf,
}

#[derive(Parser, Debug)]
pub(crate) struct ListOpts {
    #[clap(long, help = "Hide repository headers")]
    pub(crate) no_header: bool,

    #[clap(value_name = "REPO_NAMEs", num_args = 1.., help = "The repository names")]
    pub(crate) repos: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct RootOpts {
    #[clap(short, long, help = "Open the folder in the GUI file manager")]
    pub(crate) open: bool,
}

#[derive(Parser, Debug)]
pub(crate) struct SearchOpts {
    #[clap(value_name = "QUERIES...", help = "The search query")]
    pub(crate) queries: Vec<String>,
}

#[cfg(debug_assertions)]
#[derive(Parser, Debug)]
pub(crate) struct CompleteOpts {
    #[clap(
        long = "completion-out-dir",
        value_name = "DIR",
        default_value = "target/completions",
        help = "Output directory of completion files",
        hide = true
    )]
    pub(crate) dest: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump_opts(dest: &str, names: Vec<&str>) -> DumpOpts {
        DumpOpts {
            dest: dest.into(),
            no_append: false,
            clear_prologue: false,
            clear: false,
            dry_run: false,
            append: false,
            names: names.into_iter().map(String::from).collect(),
        }
    }

    /// Resolves every name to itself, as if each named a boilerplate of its own.
    fn plain(name: &str) -> Vec<String> {
        vec![name.to_lowercase()]
    }

    /// Resolves the names the way the repository `default` of `github/gitignore` does, where
    /// `AL.gitignore` and `Global/AL.gitignore` share a name and `macOS` lives under `Global/`.
    fn github(name: &str) -> Vec<String> {
        let key = match name.to_lowercase().as_str() {
            "al" | "default/al" => "default/al",
            "global/al" | "default/global/al" => "default/global/al",
            "macos" | "global/macos" | "default/global/macos" => "default/global/macos",
            "go" | "default/go" => "default/go",
            _ => return vec![],
        };
        vec![key.to_string()]
    }

    fn merged(
        opts: &DumpOpts,
        current: Vec<&str>,
        resolve: fn(&str) -> Vec<String>,
    ) -> Vec<String> {
        let current = current.into_iter().map(String::from).collect();
        opts.names_with(current, resolve)
            .iter()
            .map(|n| n.to_string())
            .collect()
    }

    #[test]
    fn dump_opts_names_drops_an_entry_by_its_file_stem() {
        let opts = dump_opts(".gitignore", vec!["-macOS"]);
        assert_eq!(
            merged(&opts, vec!["default/Global/macOS", "default/Go"], github),
            vec!["default/Go"]
        );
    }

    #[test]
    fn dump_opts_names_drops_only_the_boilerplate_the_name_resolves_to() {
        let opts = dump_opts(".gitignore", vec!["-AL"]);
        assert_eq!(
            merged(&opts, vec!["default/AL", "default/Global/AL"], github),
            vec!["default/Global/AL"]
        );
        let opts = dump_opts(".gitignore", vec!["-Global/AL"]);
        assert_eq!(
            merged(&opts, vec!["default/AL", "default/Global/AL"], github),
            vec!["default/AL"]
        );
    }

    #[test]
    fn dump_opts_names_does_not_list_a_boilerplate_twice() {
        let opts = dump_opts(".gitignore", vec!["Go", "go"]);
        assert_eq!(
            merged(&opts, vec!["default/Go"], github),
            vec!["default/Go"]
        );
    }

    #[test]
    fn dump_opts_names_keeps_a_name_resolving_to_nothing() {
        let opts = dump_opts(".gitignore", vec!["Nope", "-Nope2"]);
        assert_eq!(
            merged(&opts, vec!["default/Go"], github),
            vec!["default/Go", "Nope"]
        );
    }

    #[test]
    fn suggestions_offer_the_entries_sharing_the_file_stem() {
        let entries = ["default/Global/AL", "default/Go"];
        assert_eq!(suggestions("AL", &entries), vec!["default/Global/AL"]);
        assert_eq!(suggestions("al", &entries), vec!["default/Global/AL"]);
    }

    #[test]
    fn suggestions_offer_the_entries_a_typo_away() {
        let entries = ["default/Rust", "default/Python", "default/Go"];
        assert_eq!(suggestions("Rsut", &entries), vec!["default/Rust"]);
        assert_eq!(suggestions("Pyhton", &entries), vec!["default/Python"]);
        assert_eq!(suggestions("Pythn", &entries), vec!["default/Python"]);
        assert!(suggestions("Java", &entries).is_empty());
    }

    #[test]
    fn suggestions_put_the_same_file_stem_first() {
        let entries = ["default/Gox", "default/Global/Go"];
        assert_eq!(
            suggestions("Go", &entries),
            vec!["default/Global/Go", "default/Gox"]
        );
    }

    #[test]
    fn edit_distance_counts_a_swap_once() {
        assert_eq!(edit_distance("rust", "rust"), 0);
        assert_eq!(edit_distance("rsut", "rust"), 1);
        assert_eq!(edit_distance("pythn", "python"), 1);
        assert_eq!(edit_distance("go", "java"), 4);
    }

    #[test]
    fn dump_opts_names_appends_to_the_current_ones() {
        let opts = dump_opts(".gitignore", vec!["java"]);
        let names = opts.names_with(vec!["Rust".into(), "Python".into()], plain);
        let names = names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
        assert_eq!(names, vec!["Rust", "Python", "java"]);
    }

    #[test]
    fn dump_opts_names_removes_the_ones_prefixed_with_a_dash() {
        let opts = dump_opts(".gitignore", vec!["-rust", "go"]);
        let names = opts.names_with(vec!["Rust".into(), "Python".into()], plain);
        let names = names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
        assert_eq!(names, vec!["Python", "go"]);
    }

    #[test]
    fn dump_opts_current_list_is_empty_without_a_gitignore() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".gitignore");
        let opts = dump_opts(&dest.to_string_lossy(), vec!["java"]);
        // appending is the default, and a missing gitignore is simply an empty one
        assert_eq!(opts.current_list_if_append().unwrap(), Vec::<String>::new());
    }

    #[test]
    fn dump_opts_current_list_is_read_when_appending() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".gitignore");
        std::fs::write(
            &dest,
            "# mine\n### Generated by Gixor (https://github.com/tamada/gixor) (Rust)\ntarget\n",
        )
        .unwrap();
        let opts = dump_opts(&dest.to_string_lossy(), vec![]);
        assert_eq!(opts.current_list_if_append().unwrap(), vec!["Rust"]);
    }

    #[test]
    fn dump_opts_current_list_is_dropped_by_no_append_and_clear() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".gitignore");
        std::fs::write(&dest, "### Rust.gitignore\ntarget\n").unwrap();

        let mut opts = dump_opts(&dest.to_string_lossy(), vec![]);
        opts.no_append = true;
        assert_eq!(opts.current_list_if_append().unwrap(), Vec::<String>::new());

        let mut opts = dump_opts(&dest.to_string_lossy(), vec![]);
        opts.clear = true;
        assert_eq!(opts.current_list_if_append().unwrap(), Vec::<String>::new());
    }

    #[test]
    fn dump_opts_clear_implies_clearing_the_prologue() {
        let mut opts = dump_opts(".gitignore", vec![]);
        assert!(!opts.should_clear_prologue());
        opts.clear_prologue = true;
        assert!(opts.should_clear_prologue());

        let mut opts = dump_opts(".gitignore", vec![]);
        opts.clear = true;
        assert!(opts.should_clear_prologue());
        assert!(opts.drop_current_entries());
    }
}
