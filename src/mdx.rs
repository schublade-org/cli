//! Constrained MDX for docs/token pages.
//!
//! Frontmatter + markdown + a small set of token components. The Rust CLI
//! compiles this to catalog pages so `serve` / `build` need no JS toolchain.
//! Not a second story format — stories stay CSF / `catalog.toml`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::catalog::{Page, PageBlock};
use crate::csf;
use crate::tokens::{steps_from_map, ColorScale, TokenSet, TokenSource};

const MDX_SUFFIX: &str = ".mdx";

pub struct DiscoveredPages {
    pub pages: Vec<Page>,
    pub files: Vec<PathBuf>,
}

pub fn discover(root: &Path, tokens: &TokenSet) -> Result<DiscoveredPages, String> {
    if !root.exists() {
        return Err(format!("docs directory not found: {}", root.display()));
    }

    let mut files = Vec::new();
    collect_mdx_files(root, &mut files)?;
    files.sort();

    let mut pages = Vec::new();
    for file in &files {
        pages.push(load_mdx_file(file, tokens)?);
    }

    Ok(DiscoveredPages { pages, files })
}

fn collect_mdx_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
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
            collect_mdx_files(&path, out)?;
            continue;
        }
        if name.ends_with(MDX_SUFFIX) {
            out.push(path);
        }
    }
    Ok(())
}

pub fn load_mdx_file(path: &Path, tokens: &TokenSet) -> Result<Page, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let file_id = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("page")
        .to_string();
    parse_mdx(&raw, &file_id, tokens).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn parse_mdx(source: &str, file_id: &str, tokens: &TokenSet) -> Result<Page, String> {
    let (meta, body) = split_frontmatter(source)?;
    let id = meta
        .get("id")
        .cloned()
        .unwrap_or_else(|| file_id.to_string());
    let title = meta.get("title").cloned().unwrap_or_else(|| human_title(&id));
    let section = meta
        .get("section")
        .cloned()
        .unwrap_or_else(|| "Foundations".into());
    let description = meta.get("description").cloned().unwrap_or_default();
    let blocks = parse_body(body, tokens)?;
    Ok(Page {
        id,
        title,
        section,
        description,
        blocks,
    })
}

fn split_frontmatter(source: &str) -> Result<(BTreeMap<String, String>, &str), String> {
    let trimmed = source.trim_start_matches('\u{feff}');
    let Some(rest) = trimmed.strip_prefix("---") else {
        return Ok((BTreeMap::new(), trimmed));
    };
    let rest = rest.strip_prefix('\n').or_else(|| rest.strip_prefix("\r\n")).unwrap_or(rest);
    let close = rest
        .find("\n---")
        .or_else(|| rest.find("\r\n---"))
        .ok_or_else(|| "unclosed MDX frontmatter".to_string())?;
    let raw_meta = &rest[..close];
    let fence = close + 1;
    let after = rest[fence..]
        .find('\n')
        .map(|rel| fence + rel + 1)
        .unwrap_or(rest.len());
    Ok((parse_frontmatter(raw_meta), &rest[after..]))
}

fn parse_frontmatter(raw: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
        map.insert(key.trim().to_string(), value);
    }
    map
}

fn parse_body(body: &str, tokens: &TokenSet) -> Result<Vec<PageBlock>, String> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < body.len() {
        while i < body.len() && body[i..].chars().next().is_some_and(char::is_whitespace) {
            i += body[i..].chars().next().unwrap().len_utf8();
        }
        if i >= body.len() {
            break;
        }
        let next = body[i..].chars().next().unwrap();
        let second = body[i + next.len_utf8()..].chars().next();
        if next == '<' && second.is_some_and(is_component_start) {
            let (name, props, consumed) = parse_jsx_tag(&body[i..])?;
            blocks.push(component_block(&name, props, tokens)?);
            i += consumed;
            continue;
        }
        if next == '#' {
            let (block, next_i) = parse_heading(body, i);
            blocks.push(block);
            i = next_i;
            continue;
        }
        let (text, next_i) = parse_paragraph(body, i);
        if !text.is_empty() {
            blocks.push(PageBlock::Paragraph { text });
        }
        i = next_i;
    }
    Ok(blocks)
}

fn is_component_start(ch: char) -> bool {
    ch.is_ascii_uppercase() || ch == '/'
}

fn parse_heading(body: &str, start: usize) -> (PageBlock, usize) {
    let mut i = start;
    let mut level = 0u8;
    while i < body.len() && body[i..].starts_with('#') && level < 6 {
        level += 1;
        i += 1;
    }
    while i < body.len() && body[i..].starts_with(' ') {
        i += 1;
    }
    let text_start = i;
    if let Some(rel) = body[i..].find('\n') {
        i += rel;
    } else {
        i = body.len();
    }
    let text = body[text_start..i].trim().to_string();
    if i < body.len() {
        i += 1;
    }
    (PageBlock::Heading { level: level.max(1), text }, i)
}

fn parse_paragraph(body: &str, start: usize) -> (String, usize) {
    let mut i = start;
    let mut lines = Vec::new();
    while i < body.len() {
        let next = body[i..].chars().next().unwrap();
        let second = body[i + next.len_utf8()..].chars().next();
        if next == '<' && second.is_some_and(is_component_start) {
            break;
        }
        if next == '#' && (i == 0 || body[..i].ends_with('\n')) {
            break;
        }
        let rel = body[i..].find('\n').unwrap_or(body.len() - i);
        let line = body[i..i + rel].trim().to_string();
        i += rel;
        if i < body.len() {
            i += 1;
        }
        if line.is_empty() {
            break;
        }
        lines.push(line);
    }
    (lines.join(" "), i)
}

fn parse_jsx_tag(source: &str) -> Result<(String, Map<String, Value>, usize), String> {
    let mut i = 0;
    let bytes = source.as_bytes();
    if bytes.first() != Some(&b'<') {
        return Err("expected JSX tag".into());
    }
    i += 1;
    while i < source.len() && source[i..].chars().next().is_some_and(char::is_whitespace) {
        i += source[i..].chars().next().unwrap().len_utf8();
    }
    let name_start = i;
    while i < source.len() {
        let ch = source[i..].chars().next().unwrap();
        if ch.is_ascii_alphanumeric() || ch == '_' {
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    let name = source[name_start..i].to_string();
    if name.is_empty() {
        return Err("expected component name".into());
    }

    let mut props = Map::new();
    loop {
        while i < source.len() && source[i..].chars().next().is_some_and(char::is_whitespace) {
            i += source[i..].chars().next().unwrap().len_utf8();
        }
        if source[i..].starts_with("/>") {
            return Ok((name, props, i + 2));
        }
        if source[i..].starts_with('>') {
            i += 1;
            if let Some(rel) = source[i..].find(&format!("</{name}>")) {
                return Ok((name.clone(), props, i + rel + name.len() + 3));
            }
            return Ok((name, props, i));
        }
        if i >= source.len() {
            return Err(format!("unclosed <{name}>"));
        }
        let key_start = i;
        while i < source.len() {
            let ch = source[i..].chars().next().unwrap();
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                i += ch.len_utf8();
            } else {
                break;
            }
        }
        let key = source[key_start..i].to_string();
        if key.is_empty() {
            return Err(format!("invalid prop in <{name}>"));
        }
        while i < source.len() && source[i..].chars().next().is_some_and(char::is_whitespace) {
            i += source[i..].chars().next().unwrap().len_utf8();
        }
        if !source[i..].starts_with('=') {
            props.insert(key, Value::Bool(true));
            continue;
        }
        i += 1;
        while i < source.len() && source[i..].chars().next().is_some_and(char::is_whitespace) {
            i += source[i..].chars().next().unwrap().len_utf8();
        }
        let (value, next) = parse_jsx_value(&source[i..])?;
        props.insert(key, value);
        i += next;
    }
}

fn parse_jsx_value(source: &str) -> Result<(Value, usize), String> {
    let first = source.chars().next().ok_or("expected prop value")?;
    if first == '"' || first == '\'' {
        let mut parser_end = 1;
        while parser_end < source.len() {
            let ch = source[parser_end..].chars().next().unwrap();
            parser_end += ch.len_utf8();
            if ch == '\\' {
                if let Some(next) = source[parser_end..].chars().next() {
                    parser_end += next.len_utf8();
                }
                continue;
            }
            if ch == first {
                let inner = source[first.len_utf8()..parser_end - first.len_utf8()].to_string();
                return Ok((Value::String(unescape(&inner)), parser_end));
            }
        }
        return Err("unterminated string prop".into());
    }
    if first == '{' {
        let close = matching_brace(source, 0).ok_or("unclosed JSX expression")?;
        let inner = source[1..close].trim();
        if inner.starts_with('{') {
            let object = csf::parse_js_object(inner)?;
            return Ok((Value::Object(object), close + 1));
        }
        if inner == "true" {
            return Ok((Value::Bool(true), close + 1));
        }
        if inner == "false" {
            return Ok((Value::Bool(false), close + 1));
        }
        return Ok((Value::String(inner.to_string()), close + 1));
    }
    let mut end = 0;
    for ch in source.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return Err("expected prop value".into());
    }
    Ok((Value::String(source[..end].to_string()), end))
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(open) != Some(&b'{') {
        return None;
    }
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
            b'\'' | b'"' => quote = Some(ch),
            b'{' => depth += 1,
            b'}' => {
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

fn unescape(value: &str) -> String {
    value.replace("\\\"", "\"").replace("\\'", "'")
}

fn component_block(
    name: &str,
    props: Map<String, Value>,
    tokens: &TokenSet,
) -> Result<PageBlock, String> {
    let source = props
        .get("source")
        .and_then(Value::as_str)
        .map(parse_source)
        .transpose()?;
    match name {
        "ColorScales" => Ok(PageBlock::ColorScales {
            source,
            scales: None,
        }),
        "ColorScale" => Ok(PageBlock::ColorScale {
            scale: color_scale_from_props(&props, tokens)?,
        }),
        "Typography" | "TypeStyles" | "TokenTable" => Ok(PageBlock::Typography { source }),
        other => Err(format!(
            "unknown MDX component <{other}> — use ColorScales, ColorScale, or Typography"
        )),
    }
}

fn parse_source(value: &str) -> Result<TokenSource, String> {
    match value {
        "css" => Ok(TokenSource::Css),
        "manual" => Ok(TokenSource::Manual),
        other => Err(format!("unknown token source '{other}'")),
    }
}

fn color_scale_from_props(props: &Map<String, Value>, tokens: &TokenSet) -> Result<ColorScale, String> {
    let name = props
        .get("name")
        .and_then(Value::as_str)
        .ok_or("ColorScale needs a name prop")?
        .to_string();
    let id = props
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| crate::tokens::slug(&name));
    if let Some(steps) = props.get("steps").and_then(Value::as_object) {
        let mut map = BTreeMap::new();
        for (key, value) in steps {
            let value = value
                .as_str()
                .ok_or_else(|| format!("ColorScale step {key} must be a string"))?;
            map.insert(key.clone(), value.to_string());
        }
        return Ok(ColorScale {
            id: id.clone(),
            name,
            steps: steps_from_map(&id, &map, "color"),
            source: TokenSource::Manual,
        });
    }
    tokens
        .colors
        .iter()
        .find(|scale| scale.id == id || scale.name.eq_ignore_ascii_case(&name))
        .cloned()
        .ok_or_else(|| format!("ColorScale '{name}' has no steps and no matching token scale"))
}

fn human_title(id: &str) -> String {
    crate::tokens::humanize(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{ColorStep, TokenSet, TokenSource};

    fn tokens() -> TokenSet {
        TokenSet {
            colors: vec![ColorScale {
                id: "yellow".into(),
                name: "Yellow".into(),
                steps: vec![ColorStep {
                    step: "50".into(),
                    value: "#fffbeb".into(),
                    token: "--color-yellow-50".into(),
                }],
                source: TokenSource::Css,
            }],
            typography: crate::tokens::TypographyTokens::default(),
        }
    }

    #[test]
    fn parses_frontmatter_and_token_components() {
        let page = parse_mdx(
            r##"---
id: colors
title: Colors
section: Foundations
---

<ColorScales source="css" />

<ColorScale name="Ink" steps={{ 50: "#f8fafc", 900: "#0f172a" }} />

<Typography />
"##,
            "colors",
            &tokens(),
        )
        .unwrap();
        assert_eq!(page.id, "colors");
        assert_eq!(page.title, "Colors");
        assert_eq!(page.section, "Foundations");
        assert_eq!(page.blocks.len(), 3, "{:#?}", page.blocks);
        match &page.blocks[0] {
            PageBlock::ColorScales { source, scales } => {
                assert_eq!(source.as_ref(), Some(&TokenSource::Css));
                assert!(scales.is_none());
            }
            other => panic!("{other:?}"),
        }
        match &page.blocks[1] {
            PageBlock::ColorScale { scale } => {
                assert_eq!(scale.id, "ink");
                assert_eq!(scale.source, TokenSource::Manual);
                assert_eq!(scale.steps.len(), 2);
                assert_eq!(scale.steps[0].token, "--color-ink-50");
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(page.blocks[2], PageBlock::Typography { .. }));
    }

    #[test]
    fn heading_and_paragraph_survive() {
        let page = parse_mdx(
            "# Colors\n\nPalette scales from tokens.css.\n",
            "colors",
            &TokenSet::default(),
        )
        .unwrap();
        assert_eq!(page.blocks.len(), 2);
        assert!(matches!(
            &page.blocks[0],
            PageBlock::Heading { level: 1, text } if text == "Colors"
        ));
        assert!(matches!(
            &page.blocks[1],
            PageBlock::Paragraph { text } if text.contains("tokens.css")
        ));
    }
}
