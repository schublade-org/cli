use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::cli::ServeArgs;

const DEFAULT_TOML: &str = include_str!("../schublade.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub a11y: A11yConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub trigger: ThemeTrigger,
    #[serde(default = "default_theme_key")]
    pub key: String,
    #[serde(default = "default_light")]
    pub light: String,
    #[serde(default = "default_dark")]
    pub dark: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeTrigger {
    DataAttribute,
    #[serde(alias = "className", alias = "class")]
    ClassName,
    #[serde(alias = "localStorage", alias = "local-storage")]
    LocalStorage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_rules")]
    pub rules: Vec<A11yRule>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum A11yRule {
    ImageAlt,
    ButtonName,
    LinkName,
    Label,
    ControlName,
}

impl Default for AppConfig {
    fn default() -> Self {
        toml::from_str(DEFAULT_TOML).expect("bundled schublade.toml must parse")
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            trigger: ThemeTrigger::default(),
            key: default_theme_key(),
            light: default_light(),
            dark: default_dark(),
        }
    }
}

impl Default for A11yConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: default_rules(),
        }
    }
}

impl Default for ThemeTrigger {
    fn default() -> Self {
        Self::DataAttribute
    }
}

impl AppConfig {
    pub fn load(path: Option<&Path>) -> Result<(Self, Option<PathBuf>), String> {
        let resolved = match path {
            Some(explicit) => Some(explicit.to_path_buf()),
            None => {
                let cwd = PathBuf::from("schublade.toml");
                if cwd.exists() {
                    Some(cwd)
                } else {
                    None
                }
            }
        };

        let config = match &resolved {
            Some(file) => {
                let raw = std::fs::read_to_string(file)
                    .map_err(|error| format!("could not read {}: {error}", file.display()))?;
                toml::from_str(&raw)
                    .map_err(|error| format!("could not parse {}: {error}", file.display()))?
            }
            None => Self::default(),
        };

        Ok((config, resolved))
    }

    pub fn apply_cli(&mut self, args: &ServeArgs) {
        if let Some(host) = &args.host {
            self.server.host = host.clone();
        }
        if let Some(port) = args.port {
            self.server.port = port;
        }
    }
}

impl ThemeTrigger {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::DataAttribute => "data-attribute",
            Self::ClassName => "class-name",
            Self::LocalStorage => "local-storage",
        }
    }
}

impl A11yRule {
    pub fn as_id(self) -> &'static str {
        match self {
            Self::ImageAlt => "image-alt",
            Self::ButtonName => "button-name",
            Self::LinkName => "link-name",
            Self::Label => "label",
            Self::ControlName => "control-name",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ImageAlt => "Images must expose an alt attribute",
            Self::ButtonName => "Buttons must have an accessible name",
            Self::LinkName => "Links must have an accessible name",
            Self::Label => "Form fields must be labelled",
            Self::ControlName => "Controls must have an accessible name",
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".into()
}

fn default_port() -> u16 {
    47291
}

fn default_theme_key() -> String {
    "data-theme".into()
}

fn default_light() -> String {
    "light".into()
}

fn default_dark() -> String {
    "dark".into()
}

fn default_true() -> bool {
    true
}

fn default_rules() -> Vec<A11yRule> {
    vec![
        A11yRule::ImageAlt,
        A11yRule::ButtonName,
        A11yRule::LinkName,
        A11yRule::Label,
        A11yRule::ControlName,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_config_parses() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 47291);
        assert_eq!(config.theme.trigger, ThemeTrigger::DataAttribute);
        assert!(config.a11y.rules.contains(&A11yRule::ImageAlt));
    }

    #[test]
    fn theme_trigger_aliases() {
        let parsed: ThemeOnly = toml::from_str("trigger = \"className\"").unwrap();
        assert_eq!(parsed.trigger, ThemeTrigger::ClassName);
        let parsed: ThemeOnly = toml::from_str("trigger = \"localStorage\"").unwrap();
        assert_eq!(parsed.trigger, ThemeTrigger::LocalStorage);
    }

    #[derive(Deserialize)]
    struct ThemeOnly {
        trigger: ThemeTrigger,
    }
}
