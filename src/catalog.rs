use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{A11yRule, ThemeConfig};
use crate::tokens::{ColorScale, TokenSet, TokenSource};

const DEFAULT_CATALOG: &str = include_str!("../catalog.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub name: String,
    #[serde(default)]
    pub stories: Vec<Story>,
    /// Docs/token pages from `*.mdx` or the token preset. Not a second story format.
    #[serde(default, skip_deserializing)]
    pub pages: Vec<Page>,
}

/// Workshop docs page. CSF + `catalog.toml` stay the story source of truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub section: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub blocks: Vec<PageBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PageBlock {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        text: String,
    },
    ColorScales {
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<TokenSource>,
        #[serde(skip_serializing_if = "Option::is_none")]
        scales: Option<Vec<ColorScale>>,
    },
    ColorScale {
        scale: ColorScale,
    },
    Typography {
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<TokenSource>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub id: String,
    pub title: String,
    /// Parent drawer in the story nav, e.g. `Button` when title is `Button / Ghost`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Row label inside a group, or the flat nav label when `group` is empty.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub item: String,
    pub section: String,
    pub description: String,
    #[serde(default)]
    pub generator: Generator,
    #[serde(default)]
    pub template: Option<String>,
    pub code: String,
    #[serde(default)]
    pub controls: Vec<Control>,
    /// React/JSX source of the imported component. Omitted from `/api/bootstrap`.
    /// Static builds copy it back into bootstrap.json for offline React mounts.
    #[serde(default, skip_serializing)]
    pub component_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_export: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_name: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Generator {
    #[default]
    Html,
    AvatarGroup,
    React,
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

impl Control {
    pub fn id(&self) -> &str {
        match self {
            Self::Select { id, .. }
            | Self::Number { id, .. }
            | Self::Boolean { id, .. }
            | Self::Text { id, .. } => id,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Select { .. } => "select",
            Self::Number { .. } => "number",
            Self::Boolean { .. } => "boolean",
            Self::Text { .. } => "text",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Select { label, .. }
            | Self::Number { label, .. }
            | Self::Boolean { label, .. }
            | Self::Text { label, .. } => label,
        }
    }

    pub fn default_display(&self) -> String {
        match self {
            Self::Select { default, .. } | Self::Text { default, .. } => default.clone(),
            Self::Number { default, .. } => default.to_string(),
            Self::Boolean { default, .. } => default.to_string(),
        }
    }

    pub fn extra_display(&self) -> String {
        match self {
            Self::Select { options, .. } => options
                .iter()
                .map(|option| option.value.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            Self::Number { min, max, .. } => match (min, max) {
                (Some(min), Some(max)) => format!("{min}–{max}"),
                (Some(min), None) => format!("≥{min}"),
                (None, Some(max)) => format!("≤{max}"),
                (None, None) => String::new(),
            },
            Self::Boolean { .. } | Self::Text { .. } => String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bootstrap {
    pub catalog: Catalog,
    #[serde(default)]
    pub tokens: TokenSet,
    pub theme: ThemeConfig,
    pub a11y: A11yBootstrap,
    pub brand: BrandBootstrap,
    /// True when the workshop was written by `schublade build` and must render offline.
    #[serde(default, rename = "static", skip_serializing_if = "is_false")]
    pub static_site: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrandBootstrap {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    pub favicon: String,
}

impl BrandBootstrap {
    pub fn live(name: impl Into<String>, has_logo: bool) -> Self {
        Self {
            name: name.into(),
            logo: has_logo.then(|| "/brand/logo".to_string()),
            favicon: "/brand/favicon".into(),
        }
    }

    pub fn static_site(
        name: impl Into<String>,
        logo_file: Option<String>,
        favicon_file: String,
    ) -> Self {
        Self {
            name: name.into(),
            logo: logo_file.map(|file| format!("./{file}")),
            favicon: format!("./{favicon_file}"),
        }
    }
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

        let mut catalog: Catalog =
            toml::from_str(&raw).map_err(|error| format!("catalog: {error}"))?;
        catalog.finalize_nav();
        Ok((catalog, resolved))
    }

    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stories: Vec::new(),
            pages: Vec::new(),
        }
    }

    pub fn merge_stories(&mut self, incoming: Vec<Story>) {
        for story in incoming {
            if let Some(existing) = self.stories.iter_mut().find(|item| item.id == story.id) {
                *existing = story;
            } else {
                self.stories.push(story);
            }
        }
        self.finalize_nav();
    }

    fn finalize_nav(&mut self) {
        for story in &mut self.stories {
            story.attach_nav_parts();
        }
    }

    pub fn story(&self, id: &str) -> Option<&Story> {
        self.stories.iter().find(|story| story.id == id)
    }

    pub fn page(&self, id: &str) -> Option<&Page> {
        self.pages.iter().find(|page| page.id == id)
    }

    pub fn attach_pages(&mut self, pages: Vec<Page>) -> Result<(), String> {
        for page in &pages {
            if !is_safe_page_id(&page.id) {
                return Err(format!("invalid page id '{}'", page.id));
            }
            if self.story(&page.id).is_some() {
                return Err(format!(
                    "page id '{}' collides with a story id — CSF/catalog stories stay unique",
                    page.id
                ));
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for page in &pages {
            if !seen.insert(page.id.as_str()) {
                return Err(format!("duplicate page id '{}'", page.id));
            }
        }
        self.pages = pages;
        Ok(())
    }
}

fn is_safe_page_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

impl Story {
    pub fn attach_nav_parts(&mut self) {
        let (group, item) = nav_parts(&self.title);
        if self.group.is_none() {
            self.group = group;
        }
        if self.item.is_empty() {
            self.item = item;
        }
    }
}

/// Split `Button / Ghost` into a group drawer (`Button`) and item (`Ghost`).
pub fn nav_parts(title: &str) -> (Option<String>, String) {
    match title.split_once(" / ") {
        Some((group, item)) if !group.trim().is_empty() && !item.trim().is_empty() => {
            (Some(group.trim().to_string()), item.trim().to_string())
        }
        _ => (None, title.trim().to_string()),
    }
}

fn is_false(value: &bool) -> bool {
    !*value
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
        assert_eq!(catalog.name, "Demo catalog");
        assert!(catalog.story("avatar-group").is_some());
        assert!(catalog.stories.iter().any(|story| story.id == "button"));
        let ghost = catalog.story("button-ghost").expect("button-ghost");
        assert_eq!(ghost.group.as_deref(), Some("Button"));
        assert_eq!(ghost.item, "Ghost");
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

    #[test]
    fn merge_replaces_matching_ids() {
        let mut catalog = Catalog::empty("Demo");
        catalog.stories.push(Story {
            id: "button".into(),
            title: "Old".into(),
            group: None,
            item: String::new(),
            section: "Components".into(),
            description: String::new(),
            generator: Generator::Html,
            template: Some("<button>old</button>".into()),
            code: "<Button />".into(),
            controls: Vec::new(),
            component_source: None,
            component_export: None,
            component_name: None,
        });
        catalog.merge_stories(vec![Story {
            id: "button".into(),
            title: "New".into(),
            group: None,
            item: String::new(),
            section: "Components".into(),
            description: String::new(),
            generator: Generator::Html,
            template: Some("<button>new</button>".into()),
            code: "<Button />".into(),
            controls: Vec::new(),
            component_source: None,
            component_export: None,
            component_name: None,
        }]);
        assert_eq!(catalog.stories.len(), 1);
        assert_eq!(catalog.story("button").unwrap().title, "New");
    }

    #[test]
    fn component_source_is_not_serialized() {
        let story = Story {
            id: "button".into(),
            title: "Button".into(),
            group: None,
            item: String::new(),
            section: "Components".into(),
            description: String::new(),
            generator: Generator::React,
            template: None,
            code: "<Button />".into(),
            controls: Vec::new(),
            component_source: Some("export function Button() {}".into()),
            component_export: Some("Button".into()),
            component_name: Some("Button".into()),
        };
        let value = serde_json::to_value(&story).unwrap();
        assert!(
            value.get("component_source").is_none(),
            "component_source must stay off the wire: {value}"
        );
        assert_eq!(value["component_export"], "Button");
    }

    #[test]
    fn nav_parts_split_group_drawers() {
        assert_eq!(
            nav_parts("Button / Ghost"),
            (Some("Button".into()), "Ghost".into())
        );
        assert_eq!(nav_parts("Accordion"), (None, "Accordion".into()));
        assert_eq!(
            nav_parts("Button /  Ghost"),
            (Some("Button".into()), "Ghost".into())
        );
    }

    #[test]
    fn catalog_titles_fill_in_nav_parts() {
        let path = std::env::temp_dir().join("schublade-nav-catalog.toml");
        std::fs::write(
            &path,
            r#"
name = "Nav"
[[stories]]
id = "button-ghost"
title = "Button / Ghost"
section = "Components"
description = ""
code = "<Button />"
template = "<button>ghost</button>"
"#,
        )
        .unwrap();
        let (catalog, _) = Catalog::load(Some(&path)).unwrap();
        let story = catalog.story("button-ghost").unwrap();
        assert_eq!(story.group.as_deref(), Some("Button"));
        assert_eq!(story.item, "Ghost");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn page_ids_must_not_collide_with_stories() {
        let mut catalog = Catalog::empty("Demo");
        catalog.stories.push(Story {
            id: "colors".into(),
            title: "Colors".into(),
            group: None,
            item: String::new(),
            section: "Components".into(),
            description: String::new(),
            generator: Generator::Html,
            template: None,
            code: String::new(),
            controls: Vec::new(),
            component_source: None,
            component_export: None,
            component_name: None,
        });
        let error = catalog
            .attach_pages(vec![Page {
                id: "colors".into(),
                title: "Colors".into(),
                section: "Foundations".into(),
                description: String::new(),
                blocks: Vec::new(),
            }])
            .unwrap_err();
        assert!(error.contains("collides"), "{error}");
    }
}
