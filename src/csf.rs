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
