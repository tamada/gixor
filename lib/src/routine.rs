use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use crate::{Error, Gixor, RepositoryManager};

use super::Result;

pub(super) fn find_target_repositories<S: AsRef<str>>(
    gixor: &Gixor,
    repository_names: Vec<S>,
) -> Result<Vec<&crate::repos::Repository>> {
    if repository_names.is_empty() {
        Ok(gixor.repositories().collect::<Vec<_>>())
    } else {
        let r = repository_names
            .iter()
            .map(|name| (name, gixor.repository(name)))
            .collect::<Vec<_>>();
        if r.iter().any(|(_, repo)| repo.is_none()) {
            let errs = r
                .iter()
                .filter(|(_, repo)| repo.is_none())
                .map(|(n, _)| Error::RepositoryNotFound(n.as_ref().to_string()))
                .collect::<Vec<_>>();
            if errs.len() == 1 {
                Err(errs.into_iter().next().unwrap())
            } else {
                Err(Error::Array(errs))
            }
        } else {
            Ok(r.into_iter()
                .filter_map(|(_, repo)| repo)
                .collect::<Vec<_>>())
        }
    }
}

pub(super) fn find_boilerplates(
    gixor: &Gixor,
    names: Vec<super::Name>,
) -> Result<Vec<super::repos::Boilerplate<'_>>> {
    let r = names
        .into_iter()
        .map(|name| gixor.find(name))
        .collect::<Vec<_>>();
    match Error::vec_result_to_result_vec(r) {
        Ok(vv) => Ok(vv.into_iter().flatten().collect::<Vec<_>>()),
        Err(e) => Err(e),
    }
}

/// Finds the entries of `.gitignore` file in the given path.
/// The given path should be a directory containing a `.gitignore` file or a `.gitignore` file directly.
/// If the `.gitignore` file is not found, returns error.
pub(super) fn entries<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let gitignore_path = find_gitignore(path);
    if !gitignore_path.exists() {
        Err(super::Error::FileNotFound(gitignore_path))
    } else {
        match std::fs::File::open(gitignore_path) {
            Err(e) => Err(super::Error::IO(e)),
            Ok(f) => {
                let reader = BufReader::new(f);
                let r = reader
                    .lines()
                    .map_while(|r| r.ok())
                    .filter_map(map_to_boilerplate_name)
                    .collect::<Vec<_>>();
                Ok(r)
            }
        }
    }
}

fn map_to_boilerplate_name(line: String) -> Option<String> {
    if line.starts_with("### ") && line.ends_with(".gitignore") {
        Some(strip_to_boilerplate_name(line))
    } else {
        None
    }
}

fn strip_to_boilerplate_name(line: String) -> String {
    let items = line.rsplit("/").collect::<Vec<_>>();
    if items.is_empty() {
        "".to_string()
    } else {
        items[0].strip_suffix(".gitignore").unwrap().to_string()
    }
}

fn find_gitignore<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if path.is_dir() {
        path.join(".gitignore")
    } else {
        path.to_path_buf()
    }
}

pub(super) fn open_dest<P: AsRef<Path>>(dest: P) -> Result<Box<dyn Write>> {
    let path = dest.as_ref().to_path_buf();
    if path == Path::new("-") {
        Ok(Box::new(std::io::stdout()))
    } else if path.is_dir() {
        match std::fs::File::create(path.join(".gitignore")) {
            Ok(f) => Ok(Box::new(f)),
            Err(e) => Err(super::Error::IO(e)),
        }
    } else {
        match std::fs::File::create(dest) {
            Ok(f) => Ok(Box::new(f)),
            Err(e) => Err(super::Error::IO(e)),
        }
    }
}

pub(super) fn dump_boilerplates_impl(
    dest: impl std::io::Write,
    boilerplates: Vec<super::repos::Boilerplate>,
    clear_flag: bool,
    base_path: &Path,
) -> Result<()> {
    log::info!(
        "dumping boilerplates {:?}",
        boilerplates.iter().map(|b| b.name()).collect::<Vec<_>>()
    );
    let mut w = std::io::BufWriter::new(dest);
    let prologue = if clear_flag { vec![] } else { load_prologue() };
    let contents = Error::vec_result_to_result_vec(
        boilerplates
            .into_iter()
            .map(|b| b.dump(base_path))
            .collect::<Vec<_>>(),
    );
    match contents {
        Ok(content) => {
            let r = prologue
                .iter()
                .chain(content.iter())
                .map(|line| writeln!(w, "{line}").map_err(super::Error::IO))
                .collect::<Vec<_>>();
            match Error::vec_result_to_result_vec(r) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

fn load_prologue() -> Vec<String> {
    match std::fs::File::open(".gitignore") {
        Ok(f) => {
            log::info!("loading prologue from .gitignore");
            let mut result = vec![];
            let reader = BufReader::new(f);
            for line in reader.lines().map_while(|r| r.ok()) {
                if line.starts_with("### ") {
                    break;
                }
                result.push(line);
            }
            result
        }
        Err(_) => vec![],
    }
}
