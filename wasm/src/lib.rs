//! Browser bindings for gixor.
//!
//! The boilerplates are compiled in, so nothing here reaches for a file system, a clone or the
//! network. The page hands over the `.gitignore` it already has and gets the new one back; what
//! to do with it afterwards is the page's business.
use gixor::{GixorFactory, Name, RepositoryManager};
use wasm_bindgen::prelude::*;

/// The names of every boilerplate carried in this build, sorted, for a picker to offer.
///
/// A name is qualified with its repository (`default/Rust`) only when more than one repository
/// is carried, since the qualifier is noise while there is nothing to disambiguate.
#[wasm_bindgen]
pub fn list_boilerplates() -> Vec<String> {
    let gixor = GixorFactory::embedded();
    let base = gixor.base_path().to_path_buf();
    let qualify = gixor.len() > 1;

    let mut names = gixor
        .repositories()
        .flat_map(|repo| {
            repo.iter(&base)
                .map(|b| {
                    if qualify {
                        b.name().to_string()
                    } else {
                        b.boilerplate_name().to_string()
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}

/// Builds a `.gitignore` from the named boilerplates.
///
/// `current` is the content of the gitignore as it stands. Whatever precedes the first
/// boilerplate in it is the reader's own and is carried over; the rest is written again from the
/// boilerplates named here. Pass an empty string to start from nothing.
///
/// A name that no build carries is an error rather than a silent omission: a gitignore missing
/// the rules someone asked for looks finished and is not.
#[wasm_bindgen]
pub fn generate(names: Vec<String>, current: &str) -> Result<String, JsError> {
    let gixor = GixorFactory::embedded();
    gixor
        .build_gitignore_with(Name::parse_all(names), current)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// The commit of the boilerplate repository this build carries, so a page can say how old the
/// boilerplates it is offering are.
#[wasm_bindgen]
pub fn snapshot_commits() -> Vec<String> {
    let gixor = GixorFactory::embedded();
    let base = gixor.base_path().to_path_buf();
    gixor
        .repositories()
        .filter_map(|repo| {
            let boilerplate = repo.iter(&base).next()?;
            let hash = boilerplate.hash(&base).ok()?;
            let hash = hash.iter().map(|b| format!("{b:02x}")).collect::<String>();
            Some(format!("{}: {hash}", repo.name))
        })
        .collect()
}
