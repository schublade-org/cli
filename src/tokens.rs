//! Design tokens for workshop docs pages.
//!
//! Four adapters. This crate implements **3** and **4** only.
//!
//! 1. **Figma MCP** (sketch) — pull a published variable collection through the
//!    Figma MCP. Not implemented. Do not add a plugin slot for it.
//! 2. **Paper MCP** (sketch) — pull a design-token sheet through the Paper MCP.
//!    Not implemented.
//! 3. **Manual** (implemented) — `[[tokens.colors]]` / `[tokens.typography]` in
//!    `schublade.toml`, and/or MDX `<ColorScale steps={{ … }} />` props.
//!    When no `*.mdx` exists, the resolved set is rendered as Colors /
//!    Typography preset pages.
//! 4. **CSS file** (implemented, Tailwind-only) — `[tokens] css = "./theme.css"`
//!    reads Tailwind `@theme` / compiled theme variables. Prefix map lives in
//!    `css_tokens.rs`. Not a generic custom-property parser. No JS toolchain
//!    for `serve` / `build`.
//!
//! CSF + `catalog.toml` stay the story source of truth. MDX is docs/token
//! pages only — not a second story format.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::catalog::{Page, PageBlock};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TokenSource {
    Manual,
    Css,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenSet {
    #[serde(default)]
    pub colors: Vec<ColorScale>,
    #[serde(default)]
    pub typography: TypographyTokens,
    /// Other Tailwind families: spacing, radius, shadow, and the rest.
    #[serde(default)]
    pub groups: Vec<TokenGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenGroup {
    pub id: String,
    pub name: String,
    pub rows: Vec<TokenRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScale {
    pub id: String,
    pub name: String,
    pub steps: Vec<ColorStep>,
    pub source: TokenSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorStep {
    pub step: String,
    pub value: String,
    pub token: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypographyTokens {
    #[serde(default)]
    pub styles: Vec<TypeStyle>,
    #[serde(default)]
    pub families: Vec<TokenRow>,
    #[serde(default)]
    pub roles: Vec<TokenRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStyle {
    pub id: String,
    pub label: String,
    pub group: String,
    #[serde(rename = "fontSize")]
    pub font_size: String,
    #[serde(rename = "lineHeight")]
    pub line_height: String,
    #[serde(rename = "fontFamily", skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(rename = "fontStyle", skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    pub source: TokenSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRow {
    pub token: String,
    pub value: String,
    pub source: TokenSource,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokensConfig {
    /// Adapter 4: Tailwind theme CSS file, relative to schublade.toml.
    #[serde(default)]
    pub css: Option<PathBuf>,
    /// Adapter 3: color scales written in schublade.toml.
    #[serde(default)]
    pub colors: Vec<ManualColorScale>,
    /// Adapter 3: type roles, families, and samples.
    #[serde(default)]
    pub typography: ManualTypography,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualColorScale {
    pub name: String,
    pub id: Option<String>,
    #[serde(default)]
    pub steps: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManualTypography {
    #[serde(default)]
    pub styles: Vec<ManualTypeStyle>,
    #[serde(default)]
    pub families: Vec<ManualTokenRow>,
    #[serde(default)]
    pub roles: Vec<ManualTokenRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualTypeStyle {
    pub id: String,
    pub label: Option<String>,
    pub group: Option<String>,
    #[serde(default, rename = "font-size", alias = "font_size")]
    pub font_size: Option<String>,
    #[serde(default, rename = "line-height", alias = "line_height")]
    pub line_height: Option<String>,
    #[serde(default, rename = "font-family", alias = "font_family")]
    pub font_family: Option<String>,
    #[serde(default, rename = "font-style", alias = "font_style")]
    pub font_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualTokenRow {
    pub token: String,
    pub value: String,
}

impl TokenSet {
    pub fn merge(&mut self, other: TokenSet) {
        for scale in other.colors {
            upsert_scale(&mut self.colors, scale);
        }
        for style in other.typography.styles {
            upsert_style(&mut self.typography.styles, style);
        }
        for row in other.typography.families {
            upsert_row(&mut self.typography.families, row);
        }
        for row in other.typography.roles {
            upsert_row(&mut self.typography.roles, row);
        }
        for group in other.groups {
            upsert_group(&mut self.groups, group);
        }
    }
}

impl TypographyTokens {
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty() && self.families.is_empty() && self.roles.is_empty()
    }
}

impl TokensConfig {
    pub fn from_manual(&self) -> TokenSet {
        let mut set = TokenSet::default();
        for scale in &self.colors {
            let id = scale.id.clone().unwrap_or_else(|| slug(&scale.name));
            set.colors.push(ColorScale {
                id: id.clone(),
                name: scale.name.clone(),
                steps: steps_from_map(&id, &scale.steps, "color"),
                source: TokenSource::Manual,
            });
        }
        for style in &self.typography.styles {
            let (group, label) =
                style_names(&style.id, style.group.as_deref(), style.label.as_deref());
            set.typography.styles.push(TypeStyle {
                id: style.id.clone(),
                label,
                group,
                font_size: style.font_size.clone().unwrap_or_default(),
                line_height: style.line_height.clone().unwrap_or_default(),
                font_family: style.font_family.clone(),
                font_style: style.font_style.clone(),
                source: TokenSource::Manual,
            });
        }
        for row in &self.typography.families {
            set.typography.families.push(TokenRow {
                token: row.token.clone(),
                value: row.value.clone(),
                source: TokenSource::Manual,
            });
        }
        for row in &self.typography.roles {
            set.typography.roles.push(TokenRow {
                token: row.token.clone(),
                value: row.value.clone(),
                source: TokenSource::Manual,
            });
        }
        set
    }
}

/// Colors / Typography / other-family pages when tokens exist and no MDX was discovered.
pub fn preset_pages(tokens: &TokenSet) -> Vec<Page> {
    let mut pages = Vec::new();
    if !tokens.colors.is_empty() {
        pages.push(Page {
            id: "colors".into(),
            title: "Colors".into(),
            section: "Foundations".into(),
            description: "Color scales from the configured token adapters.".into(),
            blocks: vec![PageBlock::ColorScales {
                source: None,
                scales: None,
            }],
        });
    }
    if !tokens.typography.is_empty() {
        pages.push(Page {
            id: "typography".into(),
            title: "Typography".into(),
            section: "Foundations".into(),
            description: "Type roles and font families from the configured token adapters.".into(),
            blocks: vec![PageBlock::Typography { source: None }],
        });
    }
    for group in &tokens.groups {
        if group.rows.is_empty() {
            continue;
        }
        pages.push(Page {
            id: group.id.clone(),
            title: group.name.clone(),
            section: "Foundations".into(),
            description: format!("{} tokens from the configured token adapters.", group.name),
            blocks: vec![PageBlock::Tokens {
                family: Some(group.id.clone()),
                source: None,
            }],
        });
    }
    pages
}

pub fn steps_from_map(
    scale_id: &str,
    steps: &BTreeMap<String, String>,
    prefix: &str,
) -> Vec<ColorStep> {
    let mut out: Vec<ColorStep> = steps
        .iter()
        .map(|(step, value)| ColorStep {
            token: format!("--{prefix}-{scale_id}-{step}"),
            step: step.clone(),
            value: value.clone(),
        })
        .collect();
    sort_steps(&mut out);
    out
}

pub fn sort_steps(steps: &mut [ColorStep]) {
    steps.sort_by(
        |left, right| match (parse_step(&left.step), parse_step(&right.step)) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => left.step.cmp(&right.step),
        },
    );
}

pub fn parse_step(step: &str) -> Option<i32> {
    step.parse().ok()
}

pub fn style_names(id: &str, group: Option<&str>, label: Option<&str>) -> (String, String) {
    let (auto_group, item) = split_type_id(id);
    let group = group.map(str::to_string).unwrap_or(auto_group);
    let label = label.map(str::to_string).unwrap_or_else(|| {
        if item.is_empty() {
            group.clone()
        } else {
            format!("{group} / {item}")
        }
    });
    (group, label)
}

pub fn split_type_id(id: &str) -> (String, String) {
    match id.split_once('-') {
        Some((group, rest)) => (humanize(group), rest.replace('-', " ")),
        None => (humanize(id), String::new()),
    }
}

pub fn humanize(value: &str) -> String {
    value
        .split(|ch: char| ch == '-' || ch == '_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn slug(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == '_' || ch.is_whitespace() {
            if !out.ends_with('-') {
                out.push('-');
            }
        }
    }
    out.trim_matches('-').to_string()
}

fn upsert_scale(scales: &mut Vec<ColorScale>, incoming: ColorScale) {
    if let Some(existing) = scales.iter_mut().find(|scale| scale.id == incoming.id) {
        *existing = incoming;
    } else {
        scales.push(incoming);
    }
}

fn upsert_style(styles: &mut Vec<TypeStyle>, incoming: TypeStyle) {
    if let Some(existing) = styles.iter_mut().find(|style| style.id == incoming.id) {
        *existing = incoming;
    } else {
        styles.push(incoming);
    }
}

fn upsert_row(rows: &mut Vec<TokenRow>, incoming: TokenRow) {
    if let Some(existing) = rows.iter_mut().find(|row| row.token == incoming.token) {
        *existing = incoming;
    } else {
        rows.push(incoming);
    }
}

fn upsert_group(groups: &mut Vec<TokenGroup>, incoming: TokenGroup) {
    if let Some(existing) = groups.iter_mut().find(|group| group.id == incoming.id) {
        for row in incoming.rows {
            upsert_row(&mut existing.rows, row);
        }
    } else {
        groups.push(incoming);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_config_builds_color_and_type_tokens() {
        let raw = r##"
[[colors]]
name = "Ink"
id = "ink"
steps = { 50 = "#f8fafc", 900 = "#0f172a" }

[[typography.styles]]
id = "display-xl"
font-size = "4.5rem"
line-height = "1.05"

[[typography.families]]
token = "--font-family-sans"
value = "Inter, sans-serif"
"##;
        let config: TokensConfig = toml::from_str(raw).unwrap();
        let set = config.from_manual();
        assert_eq!(set.colors.len(), 1);
        assert_eq!(set.colors[0].id, "ink");
        assert_eq!(set.colors[0].source, TokenSource::Manual);
        assert_eq!(set.colors[0].steps[0].token, "--color-ink-50");
        assert_eq!(set.typography.styles[0].label, "Display / xl");
        assert_eq!(set.typography.families[0].token, "--font-family-sans");
    }

    #[test]
    fn preset_pages_follow_resolved_tokens() {
        let mut set = TokenSet::default();
        assert!(preset_pages(&set).is_empty());
        set.colors.push(ColorScale {
            id: "yellow".into(),
            name: "Yellow".into(),
            steps: vec![ColorStep {
                step: "50".into(),
                value: "#fff".into(),
                token: "--color-yellow-50".into(),
            }],
            source: TokenSource::Css,
        });
        set.groups.push(TokenGroup {
            id: "spacing".into(),
            name: "Spacing".into(),
            rows: vec![TokenRow {
                token: "--spacing-4".into(),
                value: "1rem".into(),
                source: TokenSource::Css,
            }],
        });
        let pages = preset_pages(&set);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].id, "colors");
        assert_eq!(pages[0].title, "Colors");
        assert_eq!(pages[1].id, "spacing");
        assert!(matches!(
            &pages[1].blocks[0],
            PageBlock::Tokens { family, .. } if family.as_deref() == Some("spacing")
        ));
    }

    #[test]
    fn css_wins_then_manual_overwrites_same_id() {
        let mut set = TokenSet::default();
        set.colors.push(ColorScale {
            id: "ink".into(),
            name: "Ink".into(),
            steps: vec![],
            source: TokenSource::Css,
        });
        set.merge(TokenSet {
            colors: vec![ColorScale {
                id: "ink".into(),
                name: "Ink override".into(),
                steps: vec![],
                source: TokenSource::Manual,
            }],
            typography: TypographyTokens::default(),
            groups: Vec::new(),
        });
        assert_eq!(set.colors.len(), 1);
        assert_eq!(set.colors[0].name, "Ink override");
        assert_eq!(set.colors[0].source, TokenSource::Manual);
    }
}
