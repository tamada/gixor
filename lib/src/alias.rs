use serde::{Deserialize, Serialize};

use crate::{AliasManager, GixorError, Name, Result};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Aliases {
    aliases: Vec<Alias>,
}

impl Aliases {
    /// Find an instance of Alias by its name.
    pub fn find<S: AsRef<str>>(&self, name: S) -> Option<&Alias> {
        let name = name.as_ref();
        self.aliases.iter().find(|&a| a.name == name)
    }

    /// Merge two instances of Aliases, avoiding duplicates by alias name, and returns a new instance.
    pub fn merge(&self, other: &Aliases) -> Self {
        let mut merged = self.aliases.clone();
        for alias in &other.aliases {
            if !merged.iter().any(|a| a.name == alias.name) {
                merged.push(alias.clone());
            }
        }
        Aliases { aliases: merged }
    }
}

impl AliasManager for Aliases {
    fn iter_aliases(&self) -> impl Iterator<Item = &Alias> {
        self.aliases.iter()
    }

    fn add_alias(&mut self, alias: Alias) -> Result<()> {
        self.aliases.push(alias);
        Ok(())
    }

    fn remove_alias<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        let name = name.as_ref();
        if let Some(pos) = self.aliases.iter().position(|a| a.name == name) {
            self.aliases.remove(pos);
            Ok(())
        } else {
            Err(GixorError::Alias(name.to_string()))
        }
    }
}

pub(super) fn extract_alias<'a>(
    config: &'a super::Config,
    name: &super::Name,
) -> Option<Vec<super::Boilerplate<'a>>> {
    match (&name.repository_name, &name.boilerplate_name) {
        (None, b_name) => extract_alias_impl(config, b_name),
        (Some(t), b_name) => {
            if t == "alias" {
                extract_alias_impl(config, b_name)
            } else {
                None
            }
        }
    }
}

fn extract_alias_impl<S: AsRef<str>>(
    config: &super::Config,
    name: S,
) -> Option<Vec<super::Boilerplate<'_>>> {
    match find_alias_impl(config, name) {
        Some(boilerplate_names) => config.find_all(boilerplate_names).ok(),
        None => None,
    }
}

fn find_alias_impl<S: AsRef<str>>(config: &super::Config, name: S) -> Option<Vec<Name>> {
    let name = name.as_ref();
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
    use crate::GixorBuilder;

    #[test]
    fn test_predefined_alias() {
        let gixor = GixorBuilder::load("../testdata/config.json").unwrap();
        let binding = gixor.config.aliases.unwrap();
        let alias = binding.find("os-list").unwrap();
        assert_eq!(alias.name, "os-list");
    }

    #[test]
    fn test_alias() {
        let gixor = GixorBuilder::load("../testdata/config.json").unwrap();
        gixor.prepare(false).unwrap();
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
        let gixor = GixorBuilder::load("../testdata/config.json").unwrap();
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
        let gixor = GixorBuilder::load("../testdata/config.json").unwrap();
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
