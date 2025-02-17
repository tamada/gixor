use std::path::PathBuf;

mod common;

#[test]
fn test_dump() {
    let gixor = match common::setup() {
        Ok(gixor) => gixor,
        Err(e) => {
            panic!("Failed to initialize Gixor: {}", e);
        }
    };
    let dest = PathBuf::from("integration/dump");
    let _ = std::fs::create_dir_all(&dest);
    let r = gixor::dump_boilerplates(
        &gixor,
        dest.clone(),
        vec![
            gixor::Name::parse("rust"),
            gixor::Name::parse("python"),
            gixor::Name::parse("c"),
        ],
    );
    assert!(r.is_ok());

    let r = gixor::list_entries(dest);
    assert!(r.is_ok());
    let entries = r.unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0], "Rust".to_string());
    assert_eq!(entries[1], "Python".to_string());
    assert_eq!(entries[2], "C".to_string());
}

#[test]
fn test_list_entries_not_found() {
    let r = gixor::list_entries("integration/not_found");
    assert!(r.is_err());
    let e = r.unwrap_err();
    assert!(matches!(e, gixor::GixorError::NotFound(_)));
}
