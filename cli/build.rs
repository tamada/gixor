//! Embed build information into the binary via environment variables.
//! This script collects enabled Cargo features, package name and version,
//! and git commit hash (if provided), then sets them as environment variables
//! to be included in the compiled binary.
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn collect_features() -> Vec<String> {
    let mut features = Vec::new();
    for (k, _) in env::vars() {
        if let Some(name) = k.strip_prefix("CARGO_FEATURE_") {
            features.push(name.to_lowercase().replace('_', "-"));
        }
    }
    features.sort();
    features
}

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
    let output = Command::new("git")
        .args(&args)
        .output()
        .unwrap_or_else(|e| {
            panic!("Failed to execute git rev-parse: {e}");
        });

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

fn embed_build_info_into_version() {
    let features = collect_features();
    let features_str = if features.is_empty() {
        "no features".to_string()
    } else {
        features.join(",")
    };
    let ver = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let target = env::var("TARGET").unwrap_or_default();
    let git_sha = extract_latest_git_sha();
    let branch = extract_git_branch();
    let rust_c_version = extract_rustc_version();

    let long = format!(
        r#"{ver} ({features_str}, {target})
Building with: {rust_c_version}
Git: {git_sha} (on branch {branch})"#
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("BUILD_LONG_VERSION.txt");
    std::fs::write(&dest, &long).expect("Unable to write build version file");
    println!("cargo:rerun-if-env-changed=GIT_SHA");
}

fn main() {
    embed_build_info_into_version();
}
