use serde::{Deserialize, Serialize};

use crate::{AliasManager, Name};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub description: String,
    pub boilerplates: Vec<Name>,
}

impl Alias {
    pub fn new(name: String, description: String, boilerplates: Vec<Name>) -> Self {
        Alias {
            name,
            description,
            boilerplates,
        }
    }
}

pub(super) fn extract_alias<'a>(
    config: &'a super::Config,
    name: &super::Name,
) -> Option<Vec<super::Boilerplate<'a>>> {
    match (&name.repository_name, &name.boilerplate_name) {
        (None, b_name) => extract_alias_impl(config, b_name.clone()),
        (Some(t), b_name) => {
            if t == "alias" {
                extract_alias_impl(config, b_name.clone())
            } else {
                None
            }
        }
    }
}

fn extract_alias_impl(config: &super::Config, name: String) -> Option<Vec<super::Boilerplate<'_>>> {
    match find_alias_impl(config, name) {
        Some(boilerplate_names) => match config.find_all(boilerplate_names) {
            Ok(boilerplates) => Some(boilerplates),
            Err(_) => None,
        },
        None => None,
    }
}

fn find_alias_impl(config: &super::Config, name: String) -> Option<Vec<Name>> {
    for alias in config.iter_aliases() {
        if alias.name == name {
            log::debug!("found alias: {}: {:?}", name, alias.boilerplates);
            return Some(alias.boilerplates.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Gixor;

    #[test]
    fn test_predefined_alias() {
        let gixor = Gixor::load("../testdata/config.json").unwrap();
        let binding = gixor.config.aliases.unwrap();
        let alias = binding.first().unwrap();
        assert_eq!(alias.name, "os-list");
    }

    #[test]
    fn test_alias() {
        let gixor = Gixor::load("../testdata/config.json").unwrap();
        let results = gixor.find(Name::from("os-list")).unwrap();
        assert_eq!(results.len(), 3);

        let b1 = results.first().unwrap();
        assert_eq!(b1.boilerplate_name(), "Linux");
        let b2 = results.get(1).unwrap();
        assert_eq!(b2.boilerplate_name(), "Windows");
        let b3 = results.get(2).unwrap();
        assert_eq!(b3.boilerplate_name(), "macOS");
    }

    #[test]
    fn test_alias_with_repository_name() {
        let gixor = Gixor::load("../testdata/config.json").unwrap();
        let results = gixor.find(Name::from("alias/os-list")).unwrap();
        assert_eq!(results.len(), 3);

        let b1 = results.first().unwrap();
        assert_eq!(b1.boilerplate_name(), "Linux");
        let b2 = results.get(1).unwrap();
        assert_eq!(b2.boilerplate_name(), "Windows");
        let b3 = results.get(2).unwrap();
        assert_eq!(b3.boilerplate_name(), "macOS");
    }

    #[test]
    fn test_nexted_alias() {
        let gixor = Gixor::load("../testdata/config.json").unwrap();
        let results = gixor.find(Name::from("my-default")).unwrap();
        assert_eq!(results.len(), 6);

        let b1 = results.first().unwrap();
        assert_eq!(b1.boilerplate_name(), "Linux");
        let b2 = results.get(1).unwrap();
        assert_eq!(b2.boilerplate_name(), "Windows");
        let b3 = results.get(2).unwrap();
        assert_eq!(b3.boilerplate_name(), "macOS");
        let b4 = results.get(3).unwrap();
        assert_eq!(b4.boilerplate_name(), "VisualStudioCode");
        let b5 = results.get(4).unwrap();
        assert_eq!(b5.boilerplate_name(), "Emacs");
        let b6 = results.get(5).unwrap();
        assert_eq!(b6.boilerplate_name(), "Vim");
    }
}
