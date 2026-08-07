use std::path::PathBuf;

use clap::{Parser, Subcommand};
use gixor::{Gixor, Name};

#[derive(Parser, Debug)]
#[command(name = "gixor", author, version)]
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

    // #[arg(long = "dry-run", help = "Do not perform the actual operation")]
    // pub(crate) dry_run: bool,
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
    List(AliasListOpts),
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

#[derive(Parser, Debug)]
pub(crate) struct AliasListOpts {
    #[clap(short = 'H', long, help = "Show header", default_value_t = true)]
    pub(crate) header: bool,
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
        Ok(self.names_with(current))
    }

    /// Merges the given names into `current` and parses the result.
    /// Split out from [`DumpOpts::names`] so the merging can be exercised on its own.
    fn names_with(&self, current: Vec<String>) -> Vec<Name> {
        let v = self.merge_names_with_add_or_remove(&self.names, current);
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

    fn merge_names_with_add_or_remove(
        &self,
        names: &Vec<String>,
        mut v: Vec<String>,
    ) -> Vec<String> {
        for name in names {
            if let Some(trunk) = name.strip_prefix("-") {
                let t = trunk.to_lowercase();
                v.retain(|item| item.to_lowercase() != t);
            } else {
                v.push(name.clone());
            }
        }
        v
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
    #[clap(short = 'H', long, help = "Show header", default_value_t = true)]
    pub(crate) header: bool,

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

    #[test]
    fn dump_opts_names_appends_to_the_current_ones() {
        let opts = dump_opts(".gitignore", vec!["java"]);
        let names = opts.names_with(vec!["Rust".into(), "Python".into()]);
        let names = names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
        assert_eq!(names, vec!["Rust", "Python", "java"]);
    }

    #[test]
    fn dump_opts_names_removes_the_ones_prefixed_with_a_dash() {
        let opts = dump_opts(".gitignore", vec!["-rust", "go"]);
        let names = opts.names_with(vec!["Rust".into(), "Python".into()]);
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
        std::fs::write(&dest, "# mine\n### Rust.gitignore\ntarget\n").unwrap();
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
