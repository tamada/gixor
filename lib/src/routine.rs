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
    let line = line.strip_prefix("### ").unwrap_or(&line);
    let items = line.rsplit("/").collect::<Vec<_>>();
    if items.is_empty() {
        "".to_string()
    } else {
        items[0].strip_suffix(".gitignore").unwrap().to_string()
    }
}

/// Resolves the path of the `.gitignore` file that `path` denotes.
/// A directory means the `.gitignore` file within it, anything else is taken as the file itself.
pub(super) fn find_gitignore<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if path.is_dir() {
        path.join(".gitignore")
    } else {
        path.to_path_buf()
    }
}

/// Builds the whole content to be written, without touching any file.
///
/// Every boilerplate is rendered before anything is handed back to the caller, so a boilerplate
/// that cannot be read makes the whole call fail with nothing written anywhere.
pub(super) fn build_content(
    boilerplates: Vec<super::repos::Boilerplate>,
    prologue: Vec<String>,
    base_path: &Path,
) -> Result<String> {
    log::info!(
        "dumping boilerplates {:?}",
        boilerplates.iter().map(|b| b.name()).collect::<Vec<_>>()
    );
    let contents = Error::vec_result_to_result_vec(
        boilerplates
            .into_iter()
            .map(|b| b.dump(base_path))
            .collect::<Vec<_>>(),
    )?;
    let mut result = String::new();
    for block in prologue.iter().chain(contents.iter()) {
        result.push_str(block);
        result.push('\n');
    }
    Ok(result)
}

/// Replaces `dest` with `content` so that a failure never leaves a half-written file behind.
///
/// The content goes to a temporary file next to `dest` first and is moved over `dest` by a
/// rename, which is atomic within a single file system. `dest` therefore keeps its previous
/// content until the new one is complete and on disk.
pub(super) fn write_atomically(dest: &Path, content: &str) -> Result<()> {
    let dir = match dest.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| ".gitignore".to_string());
    let temp = dir.join(format!(".{name}.gixor-{}", std::process::id()));

    let result = write_temp_then_rename(&temp, dest, content);
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result.map_err(super::Error::IO)
}

fn write_temp_then_rename(temp: &Path, dest: &Path, content: &str) -> std::io::Result<()> {
    let mut f = std::fs::File::create(temp)?;
    f.write_all(content.as_bytes())?;
    // BufWriter's Drop discards flush errors, so the flush is done explicitly and checked,
    // and sync_all makes sure the bytes reach the disk before dest starts pointing at them.
    f.flush()?;
    f.sync_all()?;
    drop(f);
    // File::create obeys the umask, which gives a fresh .gitignore the usual 0644. An existing
    // file may carry something else, and that is worth keeping.
    if let Ok(metadata) = std::fs::metadata(dest) {
        let _ = std::fs::set_permissions(temp, metadata.permissions());
    }
    std::fs::rename(temp, dest)
}

/// Reads the part of `path` that precedes the first boilerplate, known as the prologue.
/// A missing or unreadable file simply has no prologue.
pub(super) fn load_prologue(path: &Path) -> Vec<String> {
    match std::fs::File::open(path) {
        Ok(f) => {
            log::info!("loading prologue from {}", path.display());
            let reader = BufReader::new(f);
            take_prologue(reader.lines().map_while(|r| r.ok()))
        }
        Err(_) => vec![],
    }
}

/// The prologue of a gitignore already held in memory, for callers that have the text rather
/// than a path to it.
pub(super) fn prologue_of(content: &str) -> Vec<String> {
    take_prologue(content.lines().map(str::to_string))
}

/// The lines up to the first boilerplate. Everything from `### ` onwards was written by gixor
/// and is about to be written again.
fn take_prologue(lines: impl Iterator<Item = String>) -> Vec<String> {
    lines.take_while(|line| !line.starts_with("### ")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_to_boilerplate_name() {
        assert_eq!(map_to_boilerplate_name("### Rust.gitignore".into()), Some("Rust".into()));
        assert_eq!(map_to_boilerplate_name("### path/to/Rust.gitignore".into()), Some("Rust".into()));
        assert_eq!(map_to_boilerplate_name("Not a boilerplate".into()), None);
    }

    #[test]
    fn test_find_gitignore() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path();
        assert_eq!(find_gitignore(dir_path), dir_path.join(".gitignore"));
        
        let file_path = dir_path.join("custom.gitignore");
        assert_eq!(find_gitignore(&file_path), file_path);
    }

    #[test]
    fn test_find_gitignore_creates_nothing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path();
        // resolving a destination must not create it, unlike the former open_dest
        assert_eq!(find_gitignore(dir_path), dir_path.join(".gitignore"));
        assert!(!dir_path.join(".gitignore").exists());
    }

    #[test]
    fn test_load_prologue_stops_at_the_first_boilerplate() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join(".gitignore");
        std::fs::write(&path, "# mine\n*.local\n### Rust.gitignore\ntarget\n").unwrap();
        assert_eq!(load_prologue(&path), vec!["# mine", "*.local"]);

        // a missing file simply has no prologue
        assert!(load_prologue(&temp_dir.path().join("absent")).is_empty());
    }

    #[test]
    fn test_write_atomically_keeps_permissions_and_leaves_no_temp_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir.path().join(".gitignore");
        std::fs::write(&dest, "old\n").unwrap();
        let before = std::fs::metadata(&dest).unwrap().permissions();

        write_atomically(&dest, "new\ncontent\n").unwrap();

        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "new\ncontent\n");
        assert_eq!(std::fs::metadata(&dest).unwrap().permissions(), before);
        let leftovers = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("gixor-"))
            .count();
        assert_eq!(leftovers, 0, "temporary file left behind");
    }

    #[test]
    fn test_write_atomically_creates_a_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir.path().join(".gitignore");
        write_atomically(&dest, "fresh\n").unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "fresh\n");
    }

    #[test]
    fn test_write_atomically_reports_a_failure_without_touching_dest() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir.path().join("absent-dir").join(".gitignore");
        assert!(write_atomically(&dest, "content\n").is_err());
        assert!(!dest.exists());
    }

    #[test]
    fn test_build_content_joins_the_prologue_and_the_blocks() {
        let prologue = vec!["# mine".to_string(), "*.local".to_string()];
        // no boilerplate is needed to pin the prologue handling down
        let r = build_content(vec![], prologue, Path::new(".")).unwrap();
        assert_eq!(r, "# mine\n*.local\n");
    }
}
