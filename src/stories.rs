use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::catalog::{Control, Generator, SelectOption, Story};

const STORY_SUFFIXES: &[&str] = &[".stories.toml", ".story.toml"];

#[derive(Debug, Clone)]
pub struct DiscoveredStories {
    pub stories: Vec<Story>,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoryFile {
    id: Option<String>,
    title: Option<String>,
    section: Option<String>,
    description: Option<String>,
    /// Path to the actual component template, relative to this file.
    component: Option<PathBuf>,
    template: Option<String>,
    code: Option<String>,
    #[serde(default)]
    generator: Generator,
    #[serde(default)]
    args: toml::map::Map<String, toml::Value>,
    #[serde(default, alias = "arg_types")]
    arg_types: toml::map::Map<String, toml::Value>,
    #[serde(default)]
    stories: Vec<NamedStory>,
}

#[derive(Debug, Deserialize)]
struct NamedStory {
    name: String,
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    #[serde(default)]
    args: toml::map::Map<String, toml::Value>,
}

#[derive(Debug, Deserialize)]
struct ArgTypeSpec {
    #[serde(default, alias = "kind")]
    control: Option<String>,
    #[serde(default, alias = "label")]
    name: Option<String>,
    #[serde(default)]
    options: Vec<ArgOption>,
    #[serde(default)]
    min: Option<i64>,
    #[serde(default)]
    max: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ArgOption {
    Value(String),
    Pair {
        value: String,
        label: Option<String>,
    },
}

impl ArgOption {
    fn into_select_option(self) -> SelectOption {
        match self {
            Self::Value(value) => {
                let label = humanize(&value);
                SelectOption { value, label }
            }
            Self::Pair { value, label } => SelectOption {
                label: label.unwrap_or_else(|| humanize(&value)),
                value,
            },
        }
    }
}

/// Walk `root` for `*.stories.toml` / `*.story.toml` and turn them into catalog stories.
pub fn discover(root: &Path) -> Result<DiscoveredStories, String> {
    if !root.exists() {
        return Err(format!("stories directory not found: {}", root.display()));
    }

    let mut files = Vec::new();
    collect_story_files(root, &mut files)?;
    files.sort();

    let mut stories = Vec::new();
    let mut seen = BTreeSet::new();

    for file in &files {
        for story in load_story_file(file)? {
            if !seen.insert(story.id.clone()) {
                return Err(format!(
                    "duplicate story id '{}' from {}",
                    story.id,
                    file.display()
                ));
            }
            stories.push(story);
        }
    }

    Ok(DiscoveredStories { stories, files })
}

fn collect_story_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|error| format!("could not read {}: {error}", dir.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("could not read {}: {error}", dir.display()))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            if matches!(name.as_ref(), "target" | "node_modules" | "dist" | "build") {
                continue;
            }
            collect_story_files(&path, out)?;
            continue;
        }

        if is_story_file(&name) {
            out.push(path);
        }
    }

    Ok(())
}

fn is_story_file(name: &str) -> bool {
    STORY_SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
}

pub fn load_story_file(path: &Path) -> Result<Vec<Story>, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let file: StoryFile =
        toml::from_str(&raw).map_err(|error| format!("{}: {error}", path.display()))?;
    file.into_stories(path)
}

impl StoryFile {
    fn into_stories(self, path: &Path) -> Result<Vec<Story>, String> {
        let file_id = self
            .id
            .clone()
            .unwrap_or_else(|| id_from_filename(path));
        let (default_section, default_title) = split_title(
            self.title
                .as_deref()
                .unwrap_or(&humanize(&file_id)),
        );
        let title_has_path = self.title.as_deref().is_some_and(|value| value.contains('/'));
        let section = self.section.unwrap_or(default_section);
        let title = if title_has_path {
            default_title
        } else {
            self.title.clone().unwrap_or(default_title)
        };
        let description = self.description.unwrap_or_default();
        let template = resolve_template(path, self.component.as_deref(), self.template.as_deref())?;
        let base_args = toml_table_to_json(&self.args);
        let arg_types = parse_arg_types(&self.arg_types, path)?;
        let controls = build_controls(&arg_types, &base_args, path)?;
        let code = self
            .code
            .unwrap_or_else(|| default_code(&title, &controls));

        if self.stories.is_empty() {
            return Ok(vec![Story {
                id: file_id,
                title,
                section,
                description,
                generator: self.generator,
                template,
                code,
                controls,
            }]);
        }

        let mut out = Vec::with_capacity(self.stories.len());
        for named in self.stories {
            let mut values = base_args.clone();
            for (key, value) in toml_table_to_json(&named.args) {
                values.insert(key, value);
            }
            let controls = overlay_control_defaults(&controls, &values);
            let story_id = named.id.unwrap_or_else(|| {
                if named.name.eq_ignore_ascii_case("default") {
                    file_id.clone()
                } else {
                    format!("{}-{}", file_id, slug(&named.name))
                }
            });
            let story_title = named
                .title
                .unwrap_or_else(|| format!("{} / {}", title, named.name));
            let story_description = named.description.unwrap_or_else(|| description.clone());
            out.push(Story {
                id: story_id,
                title: story_title,
                section: section.clone(),
                description: story_description,
                generator: self.generator,
                template: template.clone(),
                code: code.clone(),
                controls,
            });
        }
        Ok(out)
    }
}

fn resolve_template(
    story_path: &Path,
    component: Option<&Path>,
    inline: Option<&str>,
) -> Result<Option<String>, String> {
    if let Some(component) = component {
        let resolved = if component.is_absolute() {
            component.to_path_buf()
        } else {
            story_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(component)
        };
        let html = std::fs::read_to_string(&resolved).map_err(|error| {
            format!(
                "{}: could not read component {}: {error}",
                story_path.display(),
                resolved.display()
            )
        })?;
        return Ok(Some(html));
    }

    Ok(inline.map(str::to_string))
}

fn parse_arg_types(
    raw: &toml::map::Map<String, toml::Value>,
    path: &Path,
) -> Result<Vec<(String, ArgTypeSpec)>, String> {
    let mut out = Vec::with_capacity(raw.len());
    for (id, value) in raw {
        let spec: ArgTypeSpec = value.clone().try_into().map_err(|error| {
            format!("{}: argTypes.{id}: {error}", path.display())
        })?;
        out.push((id.clone(), spec));
    }
    Ok(out)
}

fn build_controls(
    arg_types: &[(String, ArgTypeSpec)],
    args: &Map<String, Value>,
    path: &Path,
) -> Result<Vec<Control>, String> {
    let mut controls = Vec::new();
    let mut seen = BTreeSet::new();

    for (id, spec) in arg_types {
        seen.insert(id.clone());
        let default = args.get(id).cloned();
        controls.push(control_from_spec(id, spec, default, path)?);
    }

    for (id, value) in args {
        if seen.contains(id) {
            continue;
        }
        controls.push(infer_control(id, value));
    }

    Ok(controls)
}

fn control_from_spec(
    id: &str,
    spec: &ArgTypeSpec,
    default: Option<Value>,
    path: &Path,
) -> Result<Control, String> {
    let kind = spec
        .control
        .as_deref()
        .or_else(|| default.as_ref().map(inferred_kind))
        .unwrap_or("text");
    let label = spec
        .name
        .clone()
        .unwrap_or_else(|| humanize(id));

    match kind {
        "text" | "string" => Ok(Control::Text {
            id: id.to_string(),
            label,
            default: default_string(default.as_ref()),
        }),
        "boolean" | "bool" | "check" | "checkbox" => Ok(Control::Boolean {
            id: id.to_string(),
            label,
            default: default.as_ref().and_then(Value::as_bool).unwrap_or(false),
        }),
        "number" | "range" => Ok(Control::Number {
            id: id.to_string(),
            label,
            min: spec.min,
            max: spec.max,
            default: default_number(default.as_ref()),
        }),
        "select" | "radio" => {
            if spec.options.is_empty() {
                return Err(format!(
                    "{}: argTypes.{id} is a select but has no options",
                    path.display()
                ));
            }
            let options: Vec<SelectOption> = spec
                .options
                .iter()
                .cloned()
                .map(ArgOption::into_select_option)
                .collect();
            let default = default_string(default.as_ref());
            let default = if default.is_empty() {
                options[0].value.clone()
            } else {
                default
            };
            Ok(Control::Select {
                id: id.to_string(),
                label,
                options,
                default,
            })
        }
        other => Err(format!(
            "{}: unknown argTypes.{id} control '{other}'",
            path.display()
        )),
    }
}

fn infer_control(id: &str, value: &Value) -> Control {
    let label = humanize(id);
    match value {
        Value::Bool(flag) => Control::Boolean {
            id: id.to_string(),
            label,
            default: *flag,
        },
        Value::Number(number) => Control::Number {
            id: id.to_string(),
            label,
            min: None,
            max: None,
            default: number.as_i64().unwrap_or(0),
        },
        _ => Control::Text {
            id: id.to_string(),
            label,
            default: default_string(Some(value)),
        },
    }
}

fn inferred_kind(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        _ => "text",
    }
}

fn overlay_control_defaults(controls: &[Control], values: &Map<String, Value>) -> Vec<Control> {
    controls
        .iter()
        .map(|control| {
            let Some(value) = values.get(control.id()) else {
                return control.clone();
            };
            match control {
                Control::Select {
                    id,
                    label,
                    options,
                    ..
                } => Control::Select {
                    id: id.clone(),
                    label: label.clone(),
                    options: options.clone(),
                    default: default_string(Some(value)),
                },
                Control::Number {
                    id,
                    label,
                    min,
                    max,
                    ..
                } => Control::Number {
                    id: id.clone(),
                    label: label.clone(),
                    min: *min,
                    max: *max,
                    default: default_number(Some(value)),
                },
                Control::Boolean { id, label, .. } => Control::Boolean {
                    id: id.clone(),
                    label: label.clone(),
                    default: value.as_bool().unwrap_or(false),
                },
                Control::Text { id, label, .. } => Control::Text {
                    id: id.clone(),
                    label: label.clone(),
                    default: default_string(Some(value)),
                },
            }
        })
        .collect()
}

fn default_code(title: &str, controls: &[Control]) -> String {
    let component = title
        .split('/')
        .next_back()
        .unwrap_or(title)
        .split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>();
    let attrs: Vec<String> = controls
        .iter()
        .map(|control| {
            let id = control.id();
            match control {
                Control::Boolean { .. } | Control::Number { .. } => {
                    format!("{id}={{{{{id}}}}}")
                }
                _ => format!("{id}=\"{{{{{id}}}}}\""),
            }
        })
        .collect();
    if attrs.is_empty() {
        format!("<{component} />")
    } else {
        format!("<{component} {} />", attrs.join(" "))
    }
}

fn toml_table_to_json(table: &toml::map::Map<String, toml::Value>) -> Map<String, Value> {
    table
        .iter()
        .map(|(key, value)| (key.clone(), toml_to_json(value)))
        .collect()
}

fn toml_to_json(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(text) => Value::String(text.clone()),
        toml::Value::Integer(number) => Value::Number((*number).into()),
        toml::Value::Float(number) => serde_json::Number::from_f64(*number)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(flag) => Value::Bool(*flag),
        toml::Value::Datetime(datetime) => Value::String(datetime.to_string()),
        toml::Value::Array(items) => Value::Array(items.iter().map(toml_to_json).collect()),
        toml::Value::Table(table) => Value::Object(toml_table_to_json(table)),
    }
}

fn default_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Bool(flag)) => flag.to_string(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn default_number(value: Option<&Value>) -> i64 {
    match value {
        Some(Value::Number(number)) => number.as_i64().unwrap_or(0),
        Some(Value::String(text)) => text.parse().unwrap_or(0),
        Some(Value::Bool(true)) => 1,
        _ => 0,
    }
}

fn id_from_filename(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("story");
    for suffix in STORY_SUFFIXES {
        if let Some(stem) = name.strip_suffix(suffix) {
            return slug(stem);
        }
    }
    slug(name)
}

fn split_title(title: &str) -> (String, String) {
    match title.rsplit_once('/') {
        Some((section, name)) => (section.trim().to_string(), name.trim().to_string()),
        None => ("Components".into(), title.trim().to_string()),
    }
}

fn slug(input: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.extend(ch.to_ascii_lowercase().to_string().chars());
        } else if !out.is_empty() {
            pending_dash = true;
        }
    }
    if out.is_empty() {
        "story".into()
    } else {
        out
    }
}

fn humanize(input: &str) -> String {
    let mut out = String::new();
    for (index, part) in input.split(|ch: char| !ch.is_ascii_alphanumeric()).enumerate() {
        if part.is_empty() {
            continue;
        }
        if index > 0 && !out.is_empty() {
            out.push(' ');
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    if out.is_empty() {
        input.to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::render::render_story;

    fn write_tree(files: &[(&str, &str)]) -> PathBuf {
        let stamp = files
            .first()
            .map(|(name, _)| name.replace(['/', '.'], "-"))
            .unwrap_or_else(|| "tree".into());
        let root = std::env::temp_dir().join(format!(
            "schublade-stories-{}-{}-{}",
            std::process::id(),
            stamp,
            files.len()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        for (rel, contents) in files {
            let path = root.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, contents).unwrap();
        }
        root
    }

    #[test]
    fn discovers_story_file_and_includes_component() {
        let root = write_tree(&[
            (
                "components/button.html",
                r#"<button class="btn" data-variant="{{variant}}" type="button">{{label}}</button>"#,
            ),
            (
                "components/button.stories.toml",
                r#"
title = "Components/Button"
description = "Primary action."
component = "./button.html"
code = """<Button variant="{{variant}}">{{label}}</Button>"""

[args]
label = "Save changes"
variant = "primary"

[argTypes.variant]
control = "select"
name = "Variant"
options = [
  { value = "primary", label = "Primary" },
  { value = "ghost", label = "Ghost" },
]
"#,
            ),
        ]);

        let discovered = discover(&root).unwrap();
        assert_eq!(discovered.files.len(), 1);
        assert_eq!(discovered.stories.len(), 1);
        let story = &discovered.stories[0];
        assert_eq!(story.id, "button");
        assert_eq!(story.title, "Button");
        assert_eq!(story.section, "Components");
        assert_eq!(story.controls.len(), 2);
        assert!(story
            .template
            .as_deref()
            .unwrap()
            .contains(r#"data-variant="{{variant}}""#));

        let rendered = render_story(story, &Value::Object(Map::new())).unwrap();
        assert!(rendered.html.contains("Save changes"));
        assert!(rendered.code.contains("variant=\"primary\""));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn infers_controls_from_args() {
        let root = write_tree(&[(
            "chip.stories.toml",
            r#"
title = "Chip"
component = "./chip.html"

[args]
label = "Design system"
selected = true
count = 3
"#,
        ), (
            "chip.html",
            r#"<span data-selected="{{selected}}">{{label}} {{count}}</span>"#,
        )]);

        let story = &discover(&root).unwrap().stories[0];
        assert_eq!(story.controls.len(), 3);
        assert!(matches!(
            story.controls.iter().find(|c| c.id() == "selected"),
            Some(Control::Boolean { default: true, .. })
        ));
        assert!(matches!(
            story.controls.iter().find(|c| c.id() == "count"),
            Some(Control::Number { default: 3, .. })
        ));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn named_variants_share_component() {
        let root = write_tree(&[(
            "button.stories.toml",
            r#"
title = "Button"
component = "./button.html"

[args]
label = "Save"
disabled = false

[[stories]]
name = "Default"

[[stories]]
name = "Disabled"
[stories.args]
disabled = true
"#,
        ), (
            "button.html",
            "<button{{disabledAttr}}>{{label}}</button>",
        )]);

        let stories = discover(&root).unwrap().stories;
        assert_eq!(stories.len(), 2);
        assert_eq!(stories[0].id, "button");
        assert_eq!(stories[1].id, "button-disabled");
        assert_eq!(stories[0].title, "Button / Default");
        match &stories[1].controls.iter().find(|c| c.id() == "disabled") {
            Some(Control::Boolean { default, .. }) => assert!(*default),
            other => panic!("expected boolean, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn catalog_fallback_merges_with_story_files() {
        let root = write_tree(&[
            (
                "catalog.toml",
                r#"
name = "Mixed"
[[stories]]
id = "chip"
title = "Chip"
section = "Components"
description = "From catalog.toml"
code = "<Chip />"
template = "<span>chip</span>"
"#,
            ),
            (
                "components/badge.stories.toml",
                r#"
title = "Badge"
template = "<span>{{label}}</span>"
[args]
label = "In review"
"#,
            ),
        ]);

        let (mut catalog, _) = Catalog::load(Some(&root.join("catalog.toml"))).unwrap();
        let discovered = discover(&root).unwrap();
        catalog.merge_stories(discovered.stories);
        assert_eq!(catalog.name, "Mixed");
        assert!(catalog.story("chip").is_some());
        assert!(catalog.story("badge").is_some());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_component_is_a_clear_error() {
        let root = write_tree(&[(
            "ghost.stories.toml",
            r#"
title = "Ghost"
component = "./missing.html"
"#,
        )]);
        let error = discover(&root).unwrap_err();
        assert!(error.contains("could not read component"), "{error}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn select_without_options_errors() {
        let root = write_tree(&[(
            "bad.stories.toml",
            r#"
title = "Bad"
template = "<div></div>"
[argTypes.tone]
control = "select"
"#,
        )]);
        let error = discover(&root).unwrap_err();
        assert!(error.contains("no options"), "{error}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn example_story_files_discover() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/story-files/components");
        let discovered = discover(&root).unwrap();
        assert!(
            discovered.files.len() >= 2,
            "expected button + badge story files"
        );
        let ids: Vec<_> = discovered.stories.iter().map(|story| story.id.as_str()).collect();
        assert!(ids.contains(&"button"), "{ids:?}");
        assert!(ids.contains(&"button-ghost"), "{ids:?}");
        assert!(ids.contains(&"button-disabled"), "{ids:?}");
        assert!(ids.contains(&"badge"), "{ids:?}");
        let button = discovered
            .stories
            .iter()
            .find(|story| story.id == "button")
            .unwrap();
        assert!(
            button
                .template
                .as_deref()
                .unwrap()
                .contains(r#"class="btn""#)
        );
    }

    #[test]
    fn slug_and_title_helpers() {
        assert_eq!(slug("Avatar Group"), "avatar-group");
        assert_eq!(split_title("Forms/Text field"), ("Forms".into(), "Text field".into()));
        assert_eq!(humanize("show-count"), "Show Count");
    }
}
