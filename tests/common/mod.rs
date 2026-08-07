// Every integration test binary compiles this module in full while using only the part it
// needs, so the helpers it does not call would otherwise be reported as dead code.
#![allow(dead_code)]

use gixor::{Error, Gixor, GixorFactory, Result};
use std::path::PathBuf;
use std::sync::OnceLock;

/// Paths are anchored on `CARGO_MANIFEST_DIR` rather than written relative to the working
/// directory, so that they stay inside the repository however the tests are started.
fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The scratch directory the integration tests clone into. Ignored by the root `.gitignore`.
pub fn integration_dir() -> PathBuf {
    manifest_dir().join("integration")
}

/// The configuration used by the tests, carrying three repositories and two aliases.
pub fn config_path() -> PathBuf {
    let path = manifest_dir().join("testdata").join("config.json");
    assert!(
        path.exists(),
        "{}: the test configuration is missing",
        path.display()
    );
    path
}

pub fn setup() -> Result<Gixor> {
    let dir = integration_dir();
    let _ = std::fs::create_dir_all(&dir);
    let config = dir.join("config.json");
    // The integration configuration is built on first use rather than committed.
    let gixor = match GixorFactory::load(&config) {
        Ok(gixor) => gixor,
        Err(_) => GixorFactory::new_at(&config),
    };

    // The tests of a binary run in parallel and share this one clone directory, so the
    // repositories are fetched once instead of once per test racing for the same path.
    static PREPARED: OnceLock<Result<()>> = OnceLock::new();
    match PREPARED.get_or_init(|| gixor.prepare(false)) {
        Ok(_) => Ok(gixor),
        Err(e) => Err(Error::Fatal(format!("failed to prepare the repositories: {e}"))),
    }
}
