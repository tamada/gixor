use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use gixor::{AliasManager, Gixor, GixorError, Name, RepositoryManager, Result};

mod cli;
mod terminal;

/// Represents the log level.
#[derive(Parser, Debug, ValueEnum, Clone, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

fn load_gixor(config_path: Option<PathBuf>) -> Result<(Gixor, bool)> {
    let mut store_flag = false;
    let mut gixor = match config_path {
        None => {
            log::trace!("no config path specified. use default configuration");
            store_flag = true;
            Gixor::default()
        }
        Some(path) => match Gixor::load(path.clone()) {
            Ok(g) => {
                log::trace!("configuration load from {}", path.display());
                g
            }
            Err(e) => return Err(e),
        },
    };
    let gixor = if gixor.is_empty() {
        log::trace!("no repositories are given. add default repository");
        store_flag = true;
        match gixor.add_repository(gixor::Repository::default()) {
            Err(e) => Err(e),
            Ok(_) => Ok(gixor),
        }
    } else {
        Ok(gixor)
    };
    match gixor {
        Ok(g) => match g.clone_all() {
            Err(e) => Err(e),
            _ => Ok((g, store_flag)),
        },
        Err(e) => Err(e),
    }
}

fn list_aliases(gixor: &Gixor) {
    use gixor::AliasManager;
    for alias in gixor.iter_aliases() {
        println!(
            "{}: {}",
            alias.name,
            alias
                .boilerplates
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

fn remove_alias(gixor: &mut Gixor, args: Vec<String>) -> Result<()> {
    use gixor::AliasManager;
    let r = args
        .iter()
        .map(|name| gixor.remove_alias(name))
        .collect::<Vec<_>>();
    merge_errors(r)
}

fn add_alias(gixor: &mut Gixor, desc: String, args: Vec<String>) -> Result<()> {
    if let Some((alias_name, alias_values)) = args.split_first() {
        let names = alias_values.iter().map(Name::parse).collect::<Vec<_>>();
        let alias = gixor::alias::Alias::new(alias_name.clone(), desc, names);
        gixor.add_alias(alias)
    } else {
        Err(GixorError::Alias(format!(
            "alias name and boilerplate names are required: {}",
            args.join(", ")
        )))
    }
}

fn perform_alias(gixor: &mut Gixor, opts: cli::AliasOpts) -> Result<Option<&Gixor>> {
    if opts.args.is_empty() {
        list_aliases(gixor);
        Ok(None)
    } else if opts.rm {
        match remove_alias(gixor, opts.args) {
            Ok(_) => Ok(Some(gixor)),
            Err(e) => Err(e),
        }
    } else {
        match add_alias(gixor, opts.description, opts.args) {
            Err(e) => Err(e),
            Ok(_) => Ok(Some(gixor)),
        }
    }
}

fn merge_errors(r: Vec<Result<()>>) -> Result<()> {
    let mut errs = vec![];
    for e in r {
        match e {
            Ok(_) => {}
            Err(e) => errs.push(e),
        }
    }
    if errs.is_empty() {
        Ok(())
    } else if errs.len() == 1 {
        Err(errs.into_iter().next().unwrap())
    } else {
        Err(GixorError::Array(errs))
    }
}

fn perform_dump(gixor: &Gixor, opts: cli::DumpOpts) -> Result<Option<&Gixor>> {
    let dest = opts.dest.clone();
    let names = opts.names.iter().map(Name::parse).collect::<Vec<_>>();
    match gixor::dump_boilerplates(gixor, dest, names) {
        Ok(_) => Ok(None),
        Err(e) => Err(e),
    }
}

fn list_each_boilerplate(repo: &gixor::Repository, base_path: &PathBuf) -> Result<Vec<String>> {
    let r = repo
        .iter(base_path)
        .map(|entry| entry.boilerplate_name().to_string())
        .collect::<Vec<_>>();
    Ok(r)
}

fn list_boilerplates(gixor: &Gixor, opts: cli::ListOpts) -> Result<Option<&Gixor>> {
    let repos = gixor::find_target_repositories(gixor, opts.repos.clone())?;
    let base_path = gixor.base_path().to_path_buf();
    let mut errs = vec![];
    for &repo in repos.iter() {
        let header = if opts.header {
            Some(repo.name.clone())
        } else {
            None
        };
        match list_each_boilerplate(repo, &base_path) {
            Err(e) => errs.push(e),
            Ok(list) => print_in_columns_if_needed(list, header),
        }
    }
    if errs.is_empty() {
        Ok(None)
    } else if errs.len() == 1 {
        Err(errs.into_iter().next().unwrap())
    } else {
        Err(GixorError::Array(errs))
    }
}

pub(crate) fn print_in_columns_if_needed(items: Vec<String>, header: Option<String>) {
    if atty::is(atty::Stream::Stdout) {
        let term = terminal::Terminal::default();
        if let Some(header) = header {
            println!("{}", term.format_header(header));
        }
        term.format_in_column(items)
            .iter()
            .for_each(|line| println!("{}", line));
    } else {
        if let Some(header) = header {
            println!("========== {} ==========", header)
        }
        for entry in items {
            println!("{}", entry);
        }
    }
}

fn list_entries(_: &Gixor, opts: cli::EntriesOpts) -> Result<Option<&Gixor>> {
    match gixor::list_entries(opts.dir) {
        Err(e) => Err(e),
        Ok(entries) => {
            print_in_columns_if_needed(entries, None);
            Ok(None)
        }
    }
}

fn show_root(gixor: &Gixor, opts: cli::RootOpts) -> Result<Option<&Gixor>> {
    let path = gixor.base_path();
    if opts.open {
        match opener::open(path) {
            Ok(_) => Ok(None),
            Err(e) => Err(GixorError::Fatal(format!(
                "failed to open {:?}: {:?}",
                path, e
            ))),
        }
    } else {
        println!("{}", path.to_string_lossy());
        Ok(None)
    }
}

fn update_repositories(gixor: &Gixor) -> Result<Option<&Gixor>> {
    match gixor.update_all() {
        Ok(_) => Ok(None),
        Err(e) => Err(e),
    }
}

fn search_boilerplates(gixor: &Gixor, opts: cli::SearchOpts) -> Result<Option<&Gixor>> {
    let names = gixor
        .iter()
        .map(|b| b.boilerplate_name().to_string())
        .filter(|name| {
            opts.queries
                .iter()
                .any(|query| name.to_lowercase().contains(query))
        })
        .collect::<Vec<_>>();
    print_in_columns_if_needed(names, None);
    Ok(None)
}

fn add_repository(gixor: &mut Gixor, opts: cli::RepoAddOpts) -> Result<Option<&Gixor>> {
    let repo = match opts.name {
        Some(name) => gixor::Repository::new_with(name, opts.url),
        None => gixor::Repository::new(opts.url),
    };
    match gixor.add_repository(repo) {
        Ok(_) => Ok(Some(gixor)),
        Err(e) => Err(e),
    }
}

fn remove_repository(gixor: &mut Gixor, opts: cli::RepoRemoveOpts) -> Result<Option<&Gixor>> {
    match gixor.remove_repository_with(opts.name, opts.keep_dir) {
        Ok(_) => Ok(Some(gixor)),
        Err(e) => Err(e),
    }
}

fn list_repositories(gixor: &Gixor) -> Result<Option<&Gixor>> {
    let base_path = gixor.base_path().to_path_buf();
    for repo in gixor.repositories() {
        println!(
            "{}: {}\n    {}/{}",
            repo.name,
            repo.url,
            base_path.display(),
            repo.name
        );
    }
    Ok(None)
}

fn perform_impl(gixor: &mut Gixor, subcmd: cli::GixorCommand, store_flag: bool) -> Result<bool> {
    use cli::GixorCommand::*;
    let r = match subcmd {
        Alias(opts) => perform_alias(gixor, opts),
        Dump(opts) => perform_dump(gixor, opts),
        Init => Ok(Some(&*gixor)),
        Entries(opts) => list_entries(gixor, opts),
        List(opts) => list_boilerplates(gixor, opts),
        Repository(opts) => {
            use cli::RepositoryOpts::*;
            match opts {
                Add(opts) => add_repository(gixor, opts),
                List => list_repositories(gixor),
                Remove(opts) => remove_repository(gixor, opts),
                Update => update_repositories(gixor),
            }
        }
        Root(opts) => show_root(gixor, opts),
        Search(opts) => search_boilerplates(gixor, opts),
        Update => update_repositories(gixor),
        #[cfg(debug_assertions)]
        CompletionFiles(opts) => gencomp::generate(opts.dest),
    };
    match r {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(store_flag),
        Err(e) => Err(e),
    }
}

fn perform(opts: cli::CliOpts) -> Result<()> {
    let (mut gixor, store_flag) = load_gixor(opts.config)?;
    match perform_impl(&mut gixor, opts.subcmd, store_flag) {
        Ok(flag) => {
            if flag {
                gixor.store()
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e),
    }
}

fn init_log(level: &LogLevel) {
    use LogLevel::*;
    match level {
        Error => std::env::set_var("RUST_LOG", "error"),
        Warn => std::env::set_var("RUST_LOG", "warn"),
        Info => std::env::set_var("RUST_LOG", "info"),
        Debug => std::env::set_var("RUST_LOG", "debug"),
        Trace => std::env::set_var("RUST_LOG", "trace"),
    };
    env_logger::try_init().unwrap_or_else(|_| {
        eprintln!("failed to initialize logger. set RUST_LOG to see logs.");
    });
    log::info!("set log level to {:?}", level);
}

#[cfg(debug_assertions)]
mod gencomp {
    use crate::cli::CliOpts;
    use gixor::{GixorError, Result};

    use clap::{Command, CommandFactory};
    use clap_complete::Shell;
    use std::path::PathBuf;

    fn generate_impl(app: &mut Command, shell: Shell, dest: PathBuf) -> Result<()> {
        log::info!("generate completion for {:?} to {:?}", shell, dest);
        if let Err(e) = std::fs::create_dir_all(dest.parent().unwrap()) {
            return Err(GixorError::IO(e));
        }
        match std::fs::File::create(dest) {
            Err(e) => Err(GixorError::IO(e)),
            Ok(mut out) => {
                clap_complete::generate(shell, app, "gixor", &mut out);
                Ok(())
            }
        }
    }

    pub fn generate<'a>(outdir: PathBuf) -> Result<Option<&'a gixor::Gixor>> {
        let shells = vec![
            (Shell::Bash, "bash/gixor"),
            (Shell::Fish, "fish/gixor"),
            (Shell::Zsh, "zsh/_gixor"),
            (Shell::Elvish, "elvish/gixor"),
            (Shell::PowerShell, "powershell/gixor"),
        ];
        let mut app = CliOpts::command();
        app.set_bin_name("gixor");
        let mut errs = vec![];
        for (shell, file) in shells {
            if let Err(e) = generate_impl(&mut app, shell, outdir.join(file)) {
                errs.push(e);
            }
        }
        if errs.is_empty() {
            Ok(None)
        } else {
            Err(GixorError::Array(errs))
        }
    }
}

fn main() -> Result<()> {
    let opts = cli::CliOpts::parse();
    init_log(&opts.log);
    perform(opts)
}

#[cfg(test)]
mod tests {
    use super::LogLevel;

    use super::*;

    #[test]
    fn test() {
        let r = cli::CliOpts::try_parse_from(vec!["gixor", "--log", "trace", "init"]);
        match r {
            Ok(opts) => assert_eq!(opts.log, LogLevel::Trace),
            Err(e) => panic!("failed to parse: {:?}", e),
        }
    }
}
