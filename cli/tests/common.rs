use gixor::{Gixor, Result};

pub fn setup() -> Result<Gixor> {
    let _ = std::fs::create_dir_all("../integration");
    let gixor = Gixor::load("../integration/config.json")?;

    gixor.update_all()?; // clone all repositories
    Ok(gixor)
}
