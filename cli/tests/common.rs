use gixor::{Gixor, GixorFactory, Result};

pub fn setup() -> Result<Gixor> {
    let _ = std::fs::create_dir_all("../integration");
    let gixor = GixorFactory::load("../integration/config.json")?;

    gixor.prepare(false)?; // clone all repositories
    Ok(gixor)
}
