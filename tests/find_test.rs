use gixor::{Gixor, Name, Result};

#[test]
fn find() -> Result<()> {
    let gixor = Gixor::load("testdata/config.json")?;
    let result = gixor.find(Name::new_of("devcontainer")).unwrap();
    assert_eq!(result.name, "devcontainer");
    assert_eq!(result.repository_name(), "tamada");

    let result = gixor.find(Name::new_of("rust")).unwrap();
    assert_eq!(result.name, "Rust");
    assert_eq!(result.repository_name(), "default");

    let result = gixor.find(Name::new("tamada", "devcontainer")).unwrap();
    assert_eq!(result.name, "devcontainer");
    assert_eq!(result.repository_name(), "tamada");

    let result = gixor.find(Name::new("default", "devcontainer"));
    assert!(result.is_none());
    Ok(())
}