use std::process::{Command, Output};

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.json"),
        r#"{
            "base-path": "boilerplates",
            "repositories": [
                {"name": "default", "url": "https://example.invalid/default", "owner": "fixture", "repo-name": "default", "path": "default"},
                {"name": "tools", "url": "https://example.invalid/tools", "owner": "fixture", "repo-name": "tools", "path": "tools"}
            ],
            "aliases": [{"name": "systems", "description": "Test alias", "boilerplates": ["default/Rust", "tools/Go"]}]
        }"#,
    )
    .unwrap();
    for (repo, name) in [("default", "Rust"), ("tools", "Go")] {
        let path = dir.path().join("boilerplates").join(repo);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join(format!("{name}.gitignore")), "target/\n").unwrap();
    }
    dir
}

fn run(dir: &tempfile::TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gixor"))
        .current_dir(dir.path())
        .arg("--config")
        .arg(dir.path().join("config.json"))
        .arg("--no-network")
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn list_shows_repository_headers_by_default() {
    let dir = fixture();
    assert_eq!(
        stdout(run(&dir, &["list"])),
        "========== default ==========\nRust\n========== tools ==========\nGo\n"
    );
}

#[test]
fn list_no_header_lists_only_boilerplate_names() {
    let dir = fixture();
    assert_eq!(stdout(run(&dir, &["list", "--no-header"])), "Rust\nGo\n");
}

#[test]
fn list_no_header_honours_repository_filter() {
    let dir = fixture();
    assert_eq!(stdout(run(&dir, &["list", "--no-header", "tools"])), "Go\n");
}

#[test]
fn list_alias_accepts_no_header() {
    let dir = fixture();
    assert_eq!(stdout(run(&dir, &["ls", "--no-header"])), "Rust\nGo\n");
}

#[test]
fn alias_list_keeps_existing_output() {
    let dir = fixture();
    for args in [vec!["alias", "list"], vec!["alias", "ls"], vec!["alias"]] {
        assert_eq!(
            stdout(run(&dir, &args)),
            "systems: default/Rust, tools/Go\n"
        );
    }
}

#[test]
fn alias_list_rejects_unused_header_options() {
    let dir = fixture();
    for flag in ["-H", "--header"] {
        let output = run(&dir, &["alias", "list", flag]);
        assert_eq!(output.status.code(), Some(2), "{flag} should be rejected");
        assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
    }
}
