use gixor::{Gixor, GixorFactory, Name};
use std::io::stdout;

fn main() -> gixor::Result<()> {
    // 1. Initialize Gixor with the default configuration.
    // The default configuration uses the official GitHub gitignore repository.
    let gixor = GixorFactory::load_or_default();

    // 2. Prepare the repositories (clone or update).
    // This requires network access unless the repositories are already cached.
    println!("Preparing repositories...");
    gixor.prepare(false)?;

    // 3. Define the boilerplates you want to include.
    // You can use simple names like "Rust" or "macOS".
    let names = Name::parse_all(vec!["Rust", "macOS", "VisualStudioCode"]);

    // 4. Dump the content to stdout.
    // The third argument 'false' means we don't want to clear existing content (if we were dumping to a file).
    println!("\n--- Combined .gitignore Content ---\n");
    gixor.dump(names, stdout(), false)?;

    Ok(())
}
