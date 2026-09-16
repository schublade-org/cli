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
    /// React/JSX source of the imported component. Omitted from `/api/bootstrap`.
    /// Static builds copy it back into bootstrap.json for offline React mounts.
    #[serde(default, skip_serializing)]
    pub component_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_export: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_name: Option<String>,
}
