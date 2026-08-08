mod common;

#[test]
fn test_dump() {
    let gixor = match common::setup() {
        Ok(gixor) => gixor,
        Err(e) => {
            panic!("Failed to initialize Gixor: {e}");
        }
    };
    let dest = common::integration_dir().join("dump");
    let _ = std::fs::create_dir_all(&dest);
    let dest_path = dest.join(".gitignore");
    let r = gixor.dump_to(
        vec![
            gixor::Name::parse("rust"),
            gixor::Name::parse("python"),
            gixor::Name::parse("c"),
        ],
        &dest_path,
        false,
    );
    log::info!("dump result: {r:?}");
    assert!(r.is_ok());

    let r = gixor::entries(&dest_path);
    assert!(r.is_ok());
    let entries = r.unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0], "Rust".to_string());
    assert_eq!(entries[1], "Python".to_string());
    assert_eq!(entries[2], "C".to_string());
}

#[test]
fn test_dump_failed() {
    let gixor = match common::setup() {
        Ok(gixor) => gixor,
        Err(e) => {
            panic!("Failed to initialize Gixor: {e}");
        }
    };
    let dest = common::integration_dir().join("dump");
    let _ = std::fs::create_dir_all(&dest);
    let dest_path = dest.join(".gitignore");
    let r = gixor.dump_to(vec![gixor::Name::parse("unknown")], &dest_path, false);
    assert!(r.is_err());
    let e = r.unwrap_err();
    assert!(matches!(e, gixor::Error::BoilerplateNotFound(_)))
}

/// A gitignore holds hand-written rules before the first boilerplate, and dumping must carry
/// them over rather than replace the file wholesale.
#[test]
fn test_dump_keeps_the_prologue() {
    let gixor = common::setup().expect("Failed to initialize Gixor");
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join(".gitignore");
    std::fs::write(&dest, "# my own rules\n*.local\n").unwrap();

    gixor
        .dump_to(vec![gixor::Name::parse("rust")], &dest, false)
        .unwrap();

    let content = std::fs::read_to_string(&dest).unwrap();
    assert!(content.starts_with("# my own rules\n*.local\n"), "{content}");
    assert_eq!(gixor::entries(&dest).unwrap(), vec!["Rust".to_string()]);
}

/// Dropping the prologue keeps the boilerplates and nothing else.
#[test]
fn test_dump_can_clear_the_prologue() {
    let gixor = common::setup().expect("Failed to initialize Gixor");
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join(".gitignore");
    std::fs::write(&dest, "# my own rules\n*.local\n").unwrap();

    gixor
        .dump_to(vec![gixor::Name::parse("rust")], &dest, true)
        .unwrap();

    let content = std::fs::read_to_string(&dest).unwrap();
    assert!(!content.contains("my own rules"), "{content}");
    assert_eq!(gixor::entries(&dest).unwrap(), vec!["Rust".to_string()]);
}

/// A failing dump must leave the destination byte for byte as it was, which is what makes a
/// mistyped boilerplate name harmless.
#[test]
fn test_dump_failure_leaves_the_destination_untouched() {
    let gixor = common::setup().expect("Failed to initialize Gixor");
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join(".gitignore");
    let original = "# my own rules\n*.local\n### Rust.gitignore\ntarget\n";
    std::fs::write(&dest, original).unwrap();

    let r = gixor.dump_to(vec![gixor::Name::parse("Rustt")], &dest, false);

    assert!(r.is_err());
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), original);
}

/// Appending is the default, so dumping into a directory without a gitignore has to create one
/// instead of failing on the missing file.
#[test]
fn test_dump_creates_a_missing_gitignore() {
    let gixor = common::setup().expect("Failed to initialize Gixor");
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join(".gitignore");

    gixor
        .dump_to(vec![gixor::Name::parse("rust")], &dest, false)
        .unwrap();

    assert_eq!(gixor::entries(&dest).unwrap(), vec!["Rust".to_string()]);
}

/// `build_gitignore` backs `--dry-run`, so it must produce what would be written while leaving
/// the destination alone.
#[test]
fn test_build_gitignore_does_not_write() {
    let gixor = common::setup().expect("Failed to initialize Gixor");
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join(".gitignore");
    std::fs::write(&dest, "# my own rules\n").unwrap();

    let content = gixor
        .build_gitignore(vec![gixor::Name::parse("rust")], &dest, false)
        .unwrap();

    assert!(content.starts_with("# my own rules\n"), "{content}");
    assert!(content.contains("### Rust.gitignore") || content.contains("/Rust.gitignore"));
    assert_eq!(
        std::fs::read_to_string(&dest).unwrap(),
        "# my own rules\n",
        "the destination must not be written"
    );
}

#[test]
fn test_list_entries_not_found() {
    let r = gixor::entries(common::integration_dir().join("not_found"));
    assert!(r.is_err());
    let e = r.unwrap_err();
    assert!(matches!(e, gixor::Error::FileNotFound(_)));
}
