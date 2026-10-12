//! Writes the long version shown by `gixor --version` to `$OUT_DIR/BUILD_LONG_VERSION.txt`,
//! which `cli.rs` includes: the version, the target, the rustc version, and the git commit and
//! branch, taken from `GIT_SHA` and `GIT_BRANCH` when they are set and from git otherwise.
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn extract_rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn run_git_command(args: Vec<&str>) -> Option<String> {
    let output = Command::new("git").args(&args).output().ok()?;

    if output.status.success() {
        let git_sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(git_sha)
    } else {
        None
    }
}

fn extract_latest_git_sha() -> String {
    env::var("GIT_SHA").unwrap_or_else(|_| {
        run_git_command(vec!["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string())
    })
}

fn extract_git_branch() -> String {
    env::var("GIT_BRANCH").unwrap_or_else(|_| {
        run_git_command(vec!["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_else(|| "unknown".to_string())
    })
}

/// Names the files that change with the commit and the branch, so that the build script reruns
/// when they do and only then.
fn watch_git_state() {
    // Asking git rather than joining ".git" by hand: the paths are relative to the package
    // directory, not to the repository, and in a worktree .git is a file rather than a directory.
    for name in ["HEAD", "refs/heads"] {
        if let Some(path) = run_git_command(vec![
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            name,
        ]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}

fn embed_build_info_into_version() {
    let ver = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let target = env::var("TARGET").unwrap_or_default();
    let git_sha = extract_latest_git_sha();
    let branch = extract_git_branch();
    let rustc_version = extract_rustc_version();

    let long = format!(
        r#"{ver} ({target})
Building with: {rustc_version}
Git: {git_sha} (on branch {branch})"#
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("BUILD_LONG_VERSION.txt");
    std::fs::write(&dest, &long).expect("Unable to write build version file");
    println!("cargo:rerun-if-env-changed=GIT_SHA");
    println!("cargo:rerun-if-env-changed=GIT_BRANCH");
    // Emitting any rerun-if instruction replaces Cargo's default of rerunning whenever a file
    // in the package changes, so the git state has to be named explicitly. Without these the
    // embedded commit is whatever it was the first time this ran, and never moves again.
    watch_git_state();
}

fn main() {
    embed_build_info_into_version();
}
