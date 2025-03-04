use serde::{Deserialize, Serialize};

use crate::Name;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub boilerplates: Vec<Name>,
}

impl Alias {
    pub fn new(name: String, boilerplates: Vec<Name>) -> Self {
        Alias {
            name,
            boilerplates,
        }
    }
}