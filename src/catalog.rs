use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{A11yRule, ThemeConfig};

const DEFAULT_CATALOG: &str = include_str!("../catalog.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub name: String,
    #[serde(default)]
    pub stories: Vec<Story>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub id: String,
    pub title: String,
    pub section: String,
    pub description: String,
    #[serde(default)]
    pub generator: Generator,
    #[serde(default)]
    pub template: Option<String>,
    pub code: String,
    #[serde(default)]
    pub controls: Vec<Control>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Generator {
    #[default]
    Html,
    AvatarGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Control {
    Select {
        id: String,
        label: String,
        options: Vec<SelectOption>,
        default: String,
    },
    Number {
        id: String,
        label: String,
        #[serde(default)]
        min: Option<i64>,
        #[serde(default)]
        max: Option<i64>,
        default: i64,
    },
    Boolean {
        id: String,
        label: String,
        default: bool,
    },
    Text {
        id: String,
        label: String,
        default: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bootstrap {
    pub catalog: Catalog,
    pub theme: ThemeConfig,
    pub a11y: A11yBootstrap,
}

#[derive(Debug, Clone, Serialize)]
pub struct A11yBootstrap {
    pub enabled: bool,
    pub rules: Vec<A11yRuleInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct A11yRuleInfo {
    pub id: String,
    pub label: String,
}

impl Catalog {
    pub fn load(path: Option<&Path>) -> Result<(Self, Option<PathBuf>), String> {
        let resolved = path.map(Path::to_path_buf);
        let raw = match &resolved {
            Some(file) => std::fs::read_to_string(file)
                .map_err(|error| format!("could not read {}: {error}", file.display()))?,
            None => DEFAULT_CATALOG.to_string(),
        };

        let catalog: Catalog =
            toml::from_str(&raw).map_err(|error| format!("catalog: {error}"))?;
        Ok((catalog, resolved))
    }

    pub fn story(&self, id: &str) -> Option<&Story> {
        self.stories.iter().find(|story| story.id == id)
    }
}

impl A11yBootstrap {
    pub fn from_config(enabled: bool, rules: &[A11yRule]) -> Self {
        Self {
            enabled,
            rules: rules
                .iter()
                .map(|rule| A11yRuleInfo {
                    id: rule.as_id().to_string(),
                    label: rule.label().to_string(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_parses() {
        let (catalog, path) = Catalog::load(None).unwrap();
        assert!(path.is_none());
        assert_eq!(catalog.name, "Aarau Designsystem");
        assert!(catalog.story("avatar-group").is_some());
        assert!(catalog.stories.iter().any(|story| story.id == "button"));
    }

    #[test]
    fn empty_catalog_is_allowed() {
        let path = std::env::temp_dir().join("schublade-empty-catalog.toml");
        std::fs::write(&path, "name = \"Empty catalog\"\n").unwrap();
        let (catalog, resolved) = Catalog::load(Some(&path)).unwrap();
        assert_eq!(resolved.as_deref(), Some(path.as_path()));
        assert_eq!(catalog.name, "Empty catalog");
        assert!(catalog.stories.is_empty());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn example_catalogs_parse() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
        let mut found = 0;
        for entry in std::fs::read_dir(&root).unwrap() {
            let dir = entry.unwrap().path();
            if !dir.is_dir() {
                continue;
            }
            let catalog_path = dir.join("catalog.toml");
            if !catalog_path.exists() {
                continue;
            }
            let (catalog, _) = Catalog::load(Some(&catalog_path)).unwrap();
            assert!(!catalog.name.is_empty(), "{}", catalog_path.display());
            found += 1;
        }
        assert!(found >= 7, "expected example catalogs, found {found}");
    }
}
