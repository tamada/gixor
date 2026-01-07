mod common;

#[test]
fn test_find_target_directories() {
    let gixor = match common::setup() {
        Ok(gixor) => gixor,
        Err(e) => {
            panic!("Failed to initialize Gixor: {e}");
        }
    };
    let r = gixor::find_target_repositories(&gixor, vec!["default"]).unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].name, "default");

    let r = gixor::find_target_repositories(&gixor, vec!["not_found"]);
    assert!(r.is_err());
    if let gixor::Error::RepositoryNotFound(name) = r.unwrap_err() {
        assert_eq!(name, "not_found");
    } else {
        panic!("Unexpected error");
    }

    let r = gixor::find_target_repositories(&gixor, Vec::<String>::new()).unwrap();
    assert_eq!(r.len(), 1);
}
