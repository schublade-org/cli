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
            let (group, item) = crate::catalog::nav_parts(&story_title);
            out.push(Story {
                id: story_id,
                title: story_title,
                group,
                item,
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

#[derive(Clone, Copy)]
enum ComponentKind {
    Html,
    React,
}

fn component_kind(path: &Path) -> ComponentKind {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "html" | "htm" => ComponentKind::Html,
        _ => ComponentKind::React,
    }
}

fn resolve_import(
    imports: &[Import],
    component: Option<&str>,
    story_path: &Path,
) -> Result<Import, String> {
    if imports.is_empty() {
        return Err(format!(
            "{}: story must import an HTML or React component",
            story_path.display()
        ));
    }
    if let Some(name) = component {
        if let Some(found) = imports.iter().find(|import| import.name == name) {
            return Ok(found.clone());
        }
    }
    if imports.len() == 1 {
        return Ok(imports[0].clone());
    }
    Err(format!(
        "{}: export default.component must match one import",
        story_path.display()
    ))
}

fn resolve_against_story(story_path: &Path, import: &Path) -> PathBuf {
    if import.is_absolute() {
        import.to_path_buf()
    } else {
        story_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(import)
    }
}

fn parse_imports(source: &str) -> Result<Vec<Import>, String> {
    let mut imports = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        if !line.starts_with("import ") {
            continue;
        }
        if let Some(import) = parse_import_line(line) {
            imports.push(import);
        }
    }
    Ok(imports)
}

fn parse_import_line(line: &str) -> Option<Import> {
    let from = line.find(" from ")?;
    let spec = line[from + 6..].trim().trim_end_matches(';').trim();
    let path = spec.trim_matches(|ch| ch == '\'' || ch == '"');
    let head = line["import ".len()..from].trim();
    let name = if let Some(inner) = head.strip_prefix('{').and_then(|value| value.strip_suffix('}'))
    {
        inner.split(',').next()?.trim().split_whitespace().next()?.to_string()
    } else {
        head.split_whitespace().next()?.to_string()
    };
    Some(Import {
        name,
        path: PathBuf::from(path),
    })
}

fn find_export_default_object(source: &str) -> Option<std::ops::Range<usize>> {
    let needle = "export default";
    let start = source.find(needle)?;
    let after = start + needle.len();
    let brace = source[after..].find('{')? + after;
    let end = matching_brace(source, brace)?;
    Some(brace..end + 1)
}

fn parse_named_exports(source: &str) -> Result<Vec<CsfVariant>, String> {
    let mut variants = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < source.len() {
        if source[i..].starts_with("export const ") {
            i += "export const ".len();
            let name_end = source[i..]
                .find(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                .map(|rel| i + rel)
                .unwrap_or(source.len());
            let name = source[i..name_end].to_string();
            i = name_end;
            while i < source.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < source.len() && bytes[i] == b'=' {
                i += 1;
            }
            while i < source.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= source.len() {
                break;
            }
            let value = if bytes[i] == b'{' {
                let end = matching_brace(source, i)
                    .ok_or_else(|| format!("unclosed object for export const {name}"))?;
                let object = parse_object(&source[i..=end])?;
                i = end + 1;
                object
            } else {
                Map::new()
            };
            let args = object_field(&value, "args").unwrap_or_default();
            variants.push(CsfVariant { name, args });
            continue;
        }
        i += 1;
    }
    Ok(variants)
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(open) != Some(&b'{') && bytes.get(open) != Some(&b'[') {
        return None;
    }
    let open_ch = bytes[open];
    let close_ch = if open_ch == b'{' { b'}' } else { b']' };
    let mut depth = 0;
    let mut i = open;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let ch = bytes[i];
        if let Some(q) = quote {
            if ch == b'\\' {
                i += 2;
                continue;
            }
            if ch == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match ch {
            b'\'' | b'"' | b'`' => quote = Some(ch),
            b if b == open_ch => depth += 1,
            b if b == close_ch => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}
