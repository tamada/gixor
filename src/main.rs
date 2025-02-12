use std::{io::Write, path::PathBuf};

use clap::Parser;
use gixor::{Gixor, GixorError, Result};
use std::io::{BufRead, BufReader};
use utils::errs_vec_to_result;

mod cli;
mod terminal;
mod utils;

fn load_gixor(config_path: Option<PathBuf>) -> Result<(Gixor, bool)> {
    let mut store_flag = false;
    let gixor = match config_path {
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
        gixor.add_repository(gixor::default_repository())
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

fn load_prologue() -> Vec<String> {
    match std::fs::File::open(".gitignore") {
        Ok(f) => {
            log::info!("loading prologue from .gitignore");
            let mut result = vec![];
            let reader = BufReader::new(f);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.starts_with("### ") {
                        break;
                    }
                    result.push(line);
                }
            }
            result
        }
        Err(_) => vec![],
    }
}

fn find_content(gixor: &Gixor, opts: cli::DumpOpts) -> Result<Vec<String>> {
    let prologue = if opts.clean { vec![] } else { load_prologue() };
    if opts.names.is_empty() {
        Ok(prologue)
    } else {
        let mut content = vec![];
        let mut errs = vec![];
        for name in opts.names.clone() {
            match gixor.dump(name.clone()) {
                Some(boilerplate) => content.push(boilerplate),
                None => errs.push(GixorError::NotFound(name)),
            }
        }
        if errs.is_empty() {
            let r = content
                .iter()
                .map(|s| s.dump())
                .collect::<Vec<_>>();
            if r.iter().any(|s| s.is_err()) {
                let r = errs
                    .into_iter()
                    .chain(r.into_iter().filter_map(|r| r.err()))
                    .collect();
                return errs_vec_to_result(r, vec![]);
            } else {
                let lines = r.into_iter().filter_map(|r| r.ok()).collect::<Vec<_>>();
                let mut result = prologue;
                result.extend(lines);
                return Ok(result);
            }
        } else {
            errs_vec_to_result(errs, vec![])
        }
    }
}

fn print_content(dest: String, content: Vec<String>) {
    let w: Box<dyn Write> = if dest == "-" {
        Box::new(std::io::stdout())
    } else {
        match std::fs::File::create(dest) {
            Ok(f) => Box::new(f),
            Err(e) => {
                log::error!("{:?}", e);
                return;
            }
        }
    };
    let mut w = std::io::BufWriter::new(w);
    for line in content {
        if let Err(e) = writeln!(w, "{}", line) {
            log::error!("{:?}", e);
            return;
        }
    }
}

fn dump_boilerplates(gixor: &Gixor, opts: cli::DumpOpts) -> Result<Option<Gixor>> {
    let dest = opts.dest.clone();
    match find_content(gixor, opts) {
        Err(e) => Err(e),
        Ok(content) => {
            print_content(dest, content);
            Ok(None)
        }
    }
}

fn strip_to_boilerplate_name(line: String) -> String {
    let items = line.rsplit("/").collect::<Vec<_>>();
    if items.len() < 1 {
        "".to_string()
    } else {
        items[0].strip_suffix(".gitignore").unwrap().to_string()
    }
}

fn find_entries(dir: PathBuf) -> Result<Vec<String>> {
    let path = dir.join(".gitignore");
    if !path.exists() {
        Err(GixorError::NotFound(path.to_string_lossy().to_string()))
    } else {
        match std::fs::File::open(path) {
            Err(e) => Err(GixorError::IO(e)),
            Ok(f) => {
                let reader = BufReader::new(f);
                let mut entries = vec![];
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if line.starts_with("### ") && line.ends_with(".gitignore") {
                            entries.push(strip_to_boilerplate_name(line));
                        }
                    }
                }
                Ok(entries)
            }
        }
    }
}

fn list_each_boilerplate(repo: &gixor::Repository, base_path: &PathBuf) -> Result<Vec<String>> {
    let r = repo
        .iter(base_path)
        .map(|entry| entry.file_stem().unwrap().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    Ok(r)
}

fn list_boilerplates(gixor: &Gixor, opts: cli::ListOpts) -> Result<Option<Gixor>> {
    let mut errs = vec![];
    let repos = if opts.repos.is_empty() {
        gixor.repositories().collect::<Vec<_>>()
    } else {
        let r = opts
            .repos
            .iter()
            .map(|name| (name, gixor.repository(name)))
            .collect::<Vec<_>>();
        r.iter()
            .filter(|(_, opts)| opts.is_none())
            .map(|(n, _)| n)
            .for_each(|&n| errs.push(GixorError::NotFound(n.clone())));
        r.iter()
            .filter(|(_, opts)| opts.is_some())
            .map(|(_n_, opts)| opts.unwrap())
            .collect::<Vec<_>>()
    };
    let base_path = gixor.base_path().to_path_buf();
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
    utils::errs_vec_to_result(errs, None)
}

pub(crate) fn print_in_columns_if_needed(items: Vec<String>, header: Option<String>) {
    if atty::is(atty::Stream::Stdout) {
        let term = terminal::Terminal::default();
        if let Some(header) = header {
            term.print_header(header);
        }
        term.print_in_column(items);
    } else {
        if let Some(header) = header {
            println!("========== {} ==========", header)
        }
        for entry in items {
            println!("{}", entry);
        }
    }
}

fn list_entries(_: &Gixor, opts: cli::EntriesOpts) -> Result<Option<Gixor>> {
    match find_entries(opts.dir) {
        Err(e) => Err(e),
        Ok(entries) => {
            print_in_columns_if_needed(entries, None);
            Ok(None)
        }
    }
}

fn show_root(gixor: &Gixor, opts: cli::RootOpts) -> Result<Option<Gixor>> {
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

fn update_repositories(gixor: &Gixor, opts: cli::UpdateOpts) -> Result<Option<Gixor>> {
    match gixor.update_all(opts.force) {
        Ok(_) => Ok(None),
        Err(e) => Err(e),
    }
}

fn search_boilerplates(gixor: &Gixor, opts: cli::SearchOpts) -> Result<Option<Gixor>> {
    let names = gixor
        .iter()
        .map(|path| path.file_stem().unwrap().to_string_lossy().to_string())
        .filter(|name| {
            opts.queries
                .iter()
                .any(|query| name.to_lowercase().contains(query))
        })
        .collect::<Vec<_>>();
    print_in_columns_if_needed(names, None);
    Ok(None)
}

fn add_repository(gixor: &Gixor, opts: cli::RepoAddOpts) -> Result<Option<Gixor>> {
    let repo = match opts.name {
        Some(name) => gixor::Repository::new_with(name, opts.url),
        None => gixor::Repository::new(opts.url),
    };
    match gixor.add_repository(repo) {
        Ok(g) => Ok(Some(g)),
        Err(e) => Err(e),
    }
}

fn remove_repository(gixor: &Gixor, opts: cli::RepoRemoveOpts) -> Result<Option<Gixor>> {
    match gixor.remove_repository_with(opts.name, opts.keep_dir) {
        Ok(g) => Ok(Some(g)),
        Err(e) => Err(e),
    }
}

fn list_repositories(gixor: &Gixor, _: cli::ListReposOpts) -> Result<Option<Gixor>> {
    let base_path = gixor.base_path().to_path_buf();
    for repo in gixor.repositories() {
        let path = base_path.join(&repo.path);
        println!(
            "{}: {}\n    {}",
            repo.name,
            repo.url,
            path.to_string_lossy().to_string()
        );
    }
    Ok(None)
}

fn perform_impl(
    gixor: Gixor,
    subcmd: cli::GixorCommand,
    store_flag: bool,
) -> Result<Option<Gixor>> {
    use cli::GixorCommand::*;
    let r = match subcmd {
        Dump(opts) => dump_boilerplates(&gixor, opts),
        Entries(opts) => list_entries(&gixor, opts),
        List(opts) => list_boilerplates(&gixor, opts),
        Repository(opts) => {
            use cli::RepositoryOpts::*;
            match opts {
                Add(opts) => add_repository(&gixor, opts),
                List(opts) => list_repositories(&gixor, opts),
                Remove(opts) => remove_repository(&gixor, opts),
                Update(opts) => update_repositories(&gixor, opts),
            }
        }
        Root(opts) => show_root(&gixor, opts),
        Search(opts) => search_boilerplates(&gixor, opts),
        Update(opts) => update_repositories(&gixor, opts),
    };
    match r {
        Ok(Some(g)) => Ok(Some(g)),
        Ok(None) => {
            if store_flag {
                Ok(Some(gixor))
            } else {
                Ok(None)
            }
        }
        Err(e) => Err(e),
    }
}

fn perform(opts: cli::CliOpts) -> Result<()> {
    let (gixor, store_flag) = load_gixor(opts.config)?;
    match perform_impl(gixor, opts.subcmd, store_flag) {
        Ok(Some(gixor)) => gixor.store(),
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

fn init_log(level: &gixor::LogLevel) {
    use gixor::LogLevel::*;
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
                clap_complete::generate(shell, app, "totebag", &mut out);
                Ok(())
            }
        }
    }

    pub fn generate(outdir: PathBuf) -> Result<()> {
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
            Ok(())
        } else {
            Err(GixorError::Array(errs))
        }
    }
}

fn main() -> Result<()> {
    let opts = cli::CliOpts::parse();
    init_log(&opts.log);
    if cfg!(debug_assertions) {
        #[cfg(debug_assertions)]
        if opts.compopts.completion {
            if let Err(e) = gencomp::generate(opts.compopts.dest.clone()) {
                return Err(e);
            }
        }
    }
    perform(opts)
}
