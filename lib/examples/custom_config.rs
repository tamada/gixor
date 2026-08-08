use gixor::{AliasManager, GixorFactory, Name, RepositoryManager};
use gixor::repos::Repository;
use gixor::aliases::Alias;
use std::path::PathBuf;

fn main() -> gixor::Result<()> {
    // 1. Create a custom configuration in a temporary location.
    // The file does not exist yet, so it is started from scratch rather than loaded.
    let config_path = PathBuf::from("custom_config.json");
    let mut gixor = GixorFactory::new_at(&config_path);

    // 2. Add a custom repository.
    println!("Adding custom repository...");
    let custom_repo = Repository::new("https://github.com/github/gitignore.git"); // Using same for example
    gixor.add_repository(custom_repo)?;

    // 3. Add an alias for common boilerplates.
    println!("Adding an alias 'my-web-stack'...");
    let web_alias = Alias::new(
        "my-web-stack".to_string(),
        "Standard boilerplates for my web projects".to_string(),
        Name::parse_all(vec!["Node", "TypeScript", "React"]),
    );
    gixor.add_alias(web_alias)?;

    // 4. List all configured repositories.
    println!("\nConfigured Repositories:");
    for repo in gixor.repositories() {
        println!(" - {}: {}", repo.name, repo.url);
    }

    // 5. List all aliases.
    println!("\nConfigured Aliases:");
    for alias in gixor.iter_aliases() {
        println!(" - {}: {} ({:?})", alias.name, alias.description, alias.boilerplates);
    }

    // 6. Save the configuration to the file.
    println!("\nSaving configuration to {}...", config_path.display());
    gixor.store()?;

    // Cleanup: Remove the temporary config file.
    if config_path.exists() {
        std::fs::remove_file(config_path).ok();
    }

    Ok(())
}
