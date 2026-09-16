use serde::{Deserialize, Serialize};

use crate::config::{A11yRule, ThemeConfig};

const DEFAULT_CATALOG: &str = include_str!("../catalog.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub name: String,
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
    pub fn load() -> Result<Self, String> {
        let raw = if std::path::Path::new("catalog.toml").exists() {
            std::fs::read_to_string("catalog.toml")
                .map_err(|error| format!("could not read catalog.toml: {error}"))?
        } else {
            DEFAULT_CATALOG.to_string()
        };

        let catalog: Catalog = toml::from_str(&raw).map_err(|error| format!("catalog: {error}"))?;
        if catalog.stories.is_empty() {
            return Err("catalog must define at least one story".into());
        }
        Ok(catalog)
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
        let catalog = Catalog::load().unwrap();
        assert_eq!(catalog.name, "Aarau Designsystem");
        assert!(catalog.story("avatar-group").is_some());
        assert!(catalog.stories.iter().any(|story| story.id == "button"));
    }
}
