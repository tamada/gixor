use gixor::{Gixor, GixorBuilder, Result};

pub fn setup() -> Result<Gixor> {
    let _ = std::fs::create_dir_all("../integration");
    let gixor = GixorBuilder::load("../integration/config.json")?;

    gixor.prepare()?; // clone all repositories
    Ok(gixor)
}
