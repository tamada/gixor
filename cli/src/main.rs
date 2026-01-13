use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

use gixor::{AliasManager, Error, Gixor, GixorFactory, Name, RepositoryManager, Result};

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

fn load_gixor(config_path: Option<PathBuf>, no_network: bool) -> Result<(Gixor, bool)> {
    let mut store_flag = false;
    let mut gixor = match config_path {
        None => {
            log::trace!("no config path specified. use default configuration");
            store_flag = true;
            Gixor::default()
        }
        Some(path) => match GixorFactory::load(path.clone()) {
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
        match gixor.add_repository(gixor::repos::Repository::default()) {
            Err(e) => Err(e),
            Ok(_) => Ok(gixor),
        }
    } else {
        Ok(gixor)
    };
    match gixor {
        Ok(g) => match g.prepare(no_network) {
            Err(e) => Err(e),
            _ => Ok((g, store_flag)),
        },
        Err(e) => Err(e),
    }
}

fn list_aliases(gixor: &Gixor) -> Result<Option<&Gixor>> {
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
    Ok(None)
}

fn remove_aliases(gixor: &mut Gixor, args: Vec<String>) -> Result<Option<&Gixor>> {
    use gixor::AliasManager;
    let r = args
        .iter()
        .map(|name| gixor.remove_alias(name))
        .collect::<Vec<_>>();
    match merge_errors(r) {
        Ok(_) => Ok(Some(gixor)),
        Err(e) => Err(e),
    }
}

fn add_alias(
    gixor: &mut Gixor,
    name: String,
    desc: String,
    args: Vec<String>,
) -> Result<Option<&Gixor>> {
    let names = args.iter().map(Name::parse).collect::<Vec<_>>();
    let alias = gixor::aliases::Alias::new(name, desc, names);
    match gixor.add_alias(alias) {
        Err(e) => Err(e),
        Ok(_) => Ok(Some(gixor)),
    }
}

fn perform_alias(gixor: &mut Gixor, opts: cli::AliasOpts) -> Result<Option<&Gixor>> {
    match opts.cmd {
        None => list_aliases(gixor),
        Some(cli::AliasCmd::List(_)) => list_aliases(gixor),
        Some(cli::AliasCmd::Add(opts)) => {
            add_alias(gixor, opts.name, opts.description, opts.boilerplates)
        }
        Some(cli::AliasCmd::Remove(opts)) => remove_aliases(gixor, opts.args),
    }
}

fn merge_errors<T>(r: Vec<Result<T>>) -> Result<Vec<T>> {
    let mut errs = vec![];
    let mut items = vec![];
    for e in r {
        match e {
            Ok(item) => items.push(item),
            Err(e) => errs.push(e),
        }
    }
    if errs.is_empty() {
        Ok(items)
    } else if errs.len() == 1 {
        Err(errs.into_iter().next().unwrap())
    } else {
        Err(Error::Array(errs))
    }
}

fn ask_impl(message: &str, choices: &[&str]) -> String {
    let answer = velvetio::choose(message, choices);
    answer.to_lowercase()
}

fn ask_overwrite<F>(path: &Path, overwrite: bool, ask_func: F) -> Option<&Path>
where
    F: FnOnce(&str, &[&str]) -> String,
{
    if !overwrite && path.exists() {
        let message = format!(
            "File {} already exists. Overwrite? [Yes/No/Stdout]",
            path.display()
        );
        let answer = ask_func(&message, &["Yes", "No", "Stdout"]);
        match answer.as_str() {
            "yes" | "y" => Some(path),
            "stdout" | "out" | "s" => Some(Path::new("-")),
            _ => None,
        }
    } else {
        Some(path)
    }
}

fn perform_dump(gixor: &Gixor, opts: cli::DumpOpts) -> Result<Option<&Gixor>> {
    let (dest, clear) = (opts.dest.clone(), opts.clear);
    let dest = gixor::normalize_dest_path(dest);
    log::debug!("resultant dest: {dest:?}");
    if let Some(new_dest) = ask_overwrite(&dest, opts.overwrite, ask_impl) {
        match opts.names() {
            Ok(names) => match gixor.dump_to(names, new_dest, clear, true) {
                Ok(_) => Ok(None),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    } else {
        Err(Error::FileAlreadyExist(dest))
    }
}

fn list_each_boilerplate(
    repo: &gixor::repos::Repository,
    base_path: &PathBuf,
) -> Result<Vec<String>> {
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
        Err(Error::Array(errs))
    }
}

pub(crate) fn print_in_columns_if_needed(items: Vec<String>, header: Option<String>) {
    if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        let term = terminal::Terminal::default();
        if let Some(header) = header {
            println!("{}", term.format_header(header));
        }
        term.format_in_column(items)
            .iter()
            .for_each(|line| println!("{line}"));
    } else {
        if let Some(header) = header {
            println!("========== {header} ==========")
        }
        for entry in items {
            println!("{entry}");
        }
    }
}

fn list_entries(_: &Gixor, opts: cli::EntriesOpts) -> Result<Option<&Gixor>> {
    match gixor::entries(opts.dir) {
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
            Err(e) => Err(Error::Fatal(format!("failed to open {path:?}: {e:?}"))),
        }
    } else {
        println!("{}", path.to_string_lossy());
        Ok(None)
    }
}

fn update_repositories(gixor: &Gixor) -> Result<Option<&Gixor>> {
    match gixor.prepare(false) {
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
        Some(name) => gixor::repos::Repository::new_with(name, opts.url),
        None => gixor::repos::Repository::new(opts.url),
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
    let (mut gixor, store_flag) = load_gixor(opts.config, opts.no_network)?;
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
    log::info!("set log level to {level:?}");
}

#[cfg(debug_assertions)]
mod gencomp {
    use crate::cli::CliOpts;
    use gixor::{Error, Result};

    use clap::{Command, CommandFactory};
    use clap_complete::Shell;
    use std::path::PathBuf;

    fn generate_impl(app: &mut Command, shell: Shell, dest: PathBuf) -> Result<()> {
        log::info!("generate completion for {shell:?} to {dest:?}");
        if let Err(e) = std::fs::create_dir_all(dest.parent().unwrap()) {
            return Err(Error::IO(e));
        }
        match std::fs::File::create(dest) {
            Err(e) => Err(Error::IO(e)),
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
        Error::to_err(None, errs)
    }
}

fn main() {
    let opts = cli::CliOpts::parse();
    init_log(&opts.log);
    if let Err(e) = perform(opts) {
        println!("Error: {e}");
        std::process::exit(1);
    }
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
            Err(e) => panic!("failed to parse: {e:?}"),
        }
    }

    #[test]
    fn test_ask_overwrite1() {
        let path = Path::new("non_existing_file_for_test.txt");
        let r = ask_overwrite(path, false, |_, _| "yes".to_string());
        assert!(r.is_some());
        assert_eq!(r.unwrap(), path);
    }

    #[test]
    fn test_ask_overwrite2() {
        let path = Path::new("../testdata/.gitignore");
        let r = ask_overwrite(path, false, |_, _| "no".to_string());
        assert!(r.is_none());
    }

    #[test]
    fn test_ask_overwrite3() {
        let path = Path::new("../testdata/.gitignore");
        let r = ask_overwrite(path, false, |_, _| "yes".to_string());
        assert!(r.is_some());
        assert_eq!(r.unwrap(), path);
    }

    #[test]
    fn test_ask_overwrite4() {
        let path = Path::new("../testdata/.gitignore");
        let r = ask_overwrite(path, false, |_, _| "s".to_string());
        assert!(r.is_some());
        assert_eq!(r.unwrap(), Path::new("-"));
    }

    #[test]
    fn test_ask_overwrite5() {
        let path = Path::new("../testdata/.gitignore");
        let r = ask_overwrite(path, true, |_, _| "no".to_string());
        assert!(r.is_some());
        assert_eq!(r.unwrap(), path);
    }
}
