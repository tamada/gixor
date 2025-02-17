use std::path::PathBuf;

use clap::{Parser, Subcommand};
use gixor::LogLevel;

#[derive(Parser, Debug)]
#[command(author, version, about, arg_required_else_help = true)]
pub(crate) struct CliOpts {
    #[clap(subcommand)]
    pub(crate) subcmd: GixorCommand,

    #[arg(short, long, help = "Specify the log level", default_value = "warn")]
    pub(crate) log: LogLevel,

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
    #[command(name = "dump", about = "Dump the boilerplates")]
    Dump(DumpOpts),
    #[command(
        name = "entries",
        about = "List the the current entries in the .gitignore file"
    )]
    Entries(EntriesOpts),
    #[command(name = "list", about = "List available boilerplates")]
    List(ListOpts),
    #[command(name = "root", about = "Show the root directory of the boilerplate")]
    Root(RootOpts),
    #[command(name = "search", about = "Search the boilerplates from the query")]
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

#[derive(Debug, Subcommand)]
pub(crate) enum RepositoryOpts {
    #[command(name = "add", about = "Add a new gitignore boilerplate repository")]
    Add(RepoAddOpts),
    #[command(
        name = "list",
        about = "List the current gitignore boilerplate repositories"
    )]
    List(ListReposOpts),
    #[command(name = "remove", about = "Remove a gitignore boilerplate repository")]
    Remove(RepoRemoveOpts),
    #[command(name = "update", about = "Update a gitignore boilerplate repository")]
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
pub(crate) struct ListReposOpts {}

#[derive(Parser, Debug)]
pub(crate) struct DumpOpts {
    #[clap(
        short,
        long,
        value_name = "DEST",
        default_value = "-",
        help = "Specify the destination directory. \"-\" means stdout."
    )]
    pub(crate) dest: String,

    #[clap(
        short,
        long,
        help = "Clear the current content of gitignore",
        default_value_t = false
    )]
    pub(crate) clean: bool,

    #[clap(value_name = "NAMES...", help = "The boilerplate names to dump.")]
    pub(crate) names: Vec<String>,
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
