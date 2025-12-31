use gixor::{GixorBuilder, Name, RepositoryManager, Result};

mod common;

/// send a request of HTTP GET to the content URL of the boilerplate,
///  and this test assumes that the response will be 200 OK.
#[tokio::test]
async fn test_clone_and_find() -> Result<()> {
    let gixor = match common::setup() {
        Ok(gixor) => gixor,
        Err(e) => {
            panic!("Failed to initialize Gixor: {e}");
        }
    };
    assert_eq!(gixor.len(), 1);
    gixor.prepare(false)?; // clone all repositories

    let result = gixor.find(Name::parse("rust")).unwrap();
    assert_eq!(result.len(), 1);
    let result = result.first().unwrap();
    assert_eq!(result.boilerplate_name(), "Rust".to_string());
    assert_eq!(result.repository_name(), "default");

    let url1 = result.content_url(gixor.base_path()).unwrap();
    let resp = reqwest::get(url1).await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    Ok(())
}

#[test]
fn test_find() {
    let gixor = GixorBuilder::load("../testdata/config.json").unwrap();
    let results = gixor.find(Name::from("devcontainer")).unwrap();
    assert_eq!(results.len(), 1);
    let result = results.first().unwrap();
    assert_eq!(result.boilerplate_name(), "devcontainer");
    assert_eq!(result.repository_name(), "tamada");

    let results = gixor.find(Name::from("rust")).unwrap();
    assert_eq!(results.len(), 1);
    let result = results.first().unwrap();
    assert_eq!(result.boilerplate_name(), "Rust");
    assert_eq!(result.repository_name(), "default");

    let results = gixor.find(Name::new("tamada", "devcontainer")).unwrap();
    assert_eq!(results.len(), 1);
    let result = results.first().unwrap();
    assert_eq!(result.boilerplate_name(), "devcontainer");
    assert_eq!(result.repository_name(), "tamada");

    let result = gixor.find(Name::new("default", "devcontainer"));
    assert!(result.is_err());
}
