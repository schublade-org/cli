use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::catalog::{Generator, Story};
use crate::stories::{
    build_controls_from_json, default_code, id_from_filename, overlay_control_defaults, slug,
    split_title,
};

#[derive(Debug, Clone)]
struct Import {
    name: String,
    path: PathBuf,
}

#[derive(Debug)]
struct CsfFile {
    imports: Vec<Import>,
    title: Option<String>,
    description: Option<String>,
    component: Option<String>,
    args: Map<String, Value>,
    arg_types: BTreeMap<String, Value>,
    variants: Vec<CsfVariant>,
}

#[derive(Debug)]
struct CsfVariant {
    name: String,
    args: Map<String, Value>,
}

pub fn load_csf_file(path: &Path) -> Result<Vec<Story>, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let file = parse_csf(&raw).map_err(|error| format!("{}: {error}", path.display()))?;
    file.into_stories(path)
}

fn parse_csf(source: &str) -> Result<CsfFile, String> {
    let stripped = strip_comments(source);
    let imports = parse_imports(&stripped)?;
    let Some(default_span) = find_export_default_object(&stripped) else {
        return Err("expected `export default { ... }`".into());
    };
    let meta = parse_object(&stripped[default_span.clone()])?;

    let title = string_field(&meta, "title");
    let description = string_field(&meta, "description");
    let component = ident_or_string_field(&meta, "component");
    let args = object_field(&meta, "args").unwrap_or_default();
    let arg_types = object_field(&meta, "argTypes")
        .or_else(|| object_field(&meta, "arg_types"))
        .map(json_object_to_map)
        .unwrap_or_default();

    let variants = parse_named_exports(&stripped)?;

    Ok(CsfFile {
        imports,
        title,
        description,
        component,
        args,
        arg_types,
        variants,
    })
}

impl CsfFile {
    fn into_stories(self, path: &Path) -> Result<Vec<Story>, String> {
        let import = resolve_import(&self.imports, self.component.as_deref(), path)?;
        let component_path = resolve_against_story(path, &import.path);
        let source = std::fs::read_to_string(&component_path).map_err(|error| {
            format!(
                "{}: could not read component {}: {error}",
                path.display(),
                component_path.display()
            )
        })?;

        let kind = component_kind(&component_path);
        let file_id = id_from_filename(path);
        let (default_section, default_title) =
            split_title(self.title.as_deref().unwrap_or(&human_title(&file_id)));
        let title_has_path = self
            .title
            .as_deref()
            .is_some_and(|value| value.contains('/'));
        let section = default_section;
        let title = if title_has_path {
            default_title
        } else {
            self.title.clone().unwrap_or(default_title)
        };
        let description = self.description.unwrap_or_default();
        let component_name = match kind {
            ComponentKind::Html if is_generic_import(&import.name) => title.clone(),
            _ => import.name.clone(),
        };
        let controls = build_controls_from_json(&self.arg_types, &self.args, path)?;
        let code = default_code(&component_name, &controls);
        let (generator, template, component_source) = match kind {
            ComponentKind::Html => (Generator::Html, Some(source), None),
            ComponentKind::React => (Generator::React, None, Some(source)),
        };

        let variants = if self.variants.is_empty() {
            vec![CsfVariant {
                name: "Default".into(),
                args: Map::new(),
            }]
        } else {
            self.variants
        };

        let multiple = variants.len() > 1
            || variants
                .first()
                .is_some_and(|variant| !variant.name.eq_ignore_ascii_case("default"));

        let mut out = Vec::with_capacity(variants.len());
        for variant in variants {
            let mut values = self.args.clone();
            for (key, value) in variant.args {
                values.insert(key, value);
            }
            let controls = overlay_control_defaults(&controls, &values);
            let story_id = if !multiple || variant.name.eq_ignore_ascii_case("default") {
                file_id.clone()
            } else {
                format!("{}-{}", file_id, slug(&variant.name))
            };
            let story_title = if multiple {
                format!("{} / {}", title, variant.name)
            } else {
                title.clone()
            };
            out.push(Story {
                id: story_id,
                title: story_title,
                section: section.clone(),
                description: description.clone(),
                generator,
                template: template.clone(),
                code: code.clone(),
                controls,
                component_source: component_source.clone(),
                component_export: Some(import.name.clone()),
                component_name: Some(component_name.clone()),
            });
        }
        Ok(out)
    }
}
