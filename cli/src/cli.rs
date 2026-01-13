use std::path::PathBuf;

use clap::{Parser, Subcommand};
use gixor::Name;

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
        short,
        long,
        help = "Keep the current entries and add the given entries.",
        default_value_t = false
    )]
    pub(crate) append: bool,

    #[clap(
        short,
        long,
        help = "Clear the current content of gitignore (clear prologue).",
        default_value_t = false
    )]
    pub(crate) clear: bool,

    #[clap(
        short,
        long,
        help = "Forcely overwrite the destination file.",
        default_value_t = false
    )]
    pub(crate) overwrite: bool,

    #[clap(value_name = "NAMES...", help = "The boilerplate names to dump.")]
    pub(crate) names: Vec<String>,
}

impl DumpOpts {
    /// Returns the target names for dumping.
    /// If append mode, firstly reads from the current `.gitignore`, otherwise, empty `vec`.
    /// Then, adds each name to the resultant `vec` of above process.
    /// Finally, convert `String` to `Name` by `Name::parse` and return it.
    pub fn names(&self) -> gixor::Result<Vec<Name>> {
        let v = self.current_list_if_append()?;
        let v = self.merge_names_with_add_or_remove(&self.names, v);
        log::debug!("parse dumping targets: {}", v.join(", "));
        Ok(Name::parse_all(v))
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
        if self.append {
            let d = if self.dest == "-" {
                String::from(".gitignore")
            } else {
                self.dest.clone()
            };
            gixor::entries(d)
        } else {
            Ok(vec![])
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

    #[test]
    fn dump_opts_names() -> gixor::Result<()> {
        let opts = DumpOpts {
            dest: "..".into(),
            append: true,
            overwrite: false,
            clear: false,
            names: vec!["java".into()],
        };
        let names = opts.names()?;
        assert_eq!(names.len(), 7);
        Ok(())
    }
}
