//! Adapter 4: load tokens from a CSS file by walking custom properties.
//!
//! Same idea as the CSF source walk: comments and strings are skipped, then
//! `--name: value;` declarations are collected. Not a full CSS engine.

use std::collections::BTreeMap;

use crate::tokens::{
    sort_steps, style_names, ColorScale, ColorStep, TokenRow, TokenSet, TokenSource, TypeStyle,
};

pub fn load_css_file(path: &std::path::Path) -> Result<TokenSet, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    parse_css_tokens(&raw)
}

pub fn parse_css_tokens(source: &str) -> Result<TokenSet, String> {
    let decls = collect_custom_properties(source);
    let resolved: BTreeMap<String, String> = resolve_all(&decls);

    let mut colors: Vec<ColorScale> = Vec::new();
    let mut font_sizes: BTreeMap<String, String> = BTreeMap::new();
    let mut line_heights: BTreeMap<String, String> = BTreeMap::new();
    let mut styles: Vec<TypeStyle> = Vec::new();
    let mut families = Vec::new();
    let mut roles = Vec::new();

    for (name, raw_value) in &decls {
        let value = resolved.get(name).unwrap_or(raw_value).clone();
        let token = format!("--{name}");

        if let Some((family, step)) = color_parts(name) {
            if let Some(scale) = colors.iter_mut().find(|scale| scale.id == family) {
                scale.steps.push(ColorStep {
                    step,
                    value,
                    token,
                });
            } else {
                colors.push(ColorScale {
                    id: family.clone(),
                    name: crate::tokens::humanize(&family),
                    steps: vec![ColorStep {
                        step,
                        value,
                        token,
                    }],
                    source: TokenSource::Css,
                });
            }
            continue;
        }

        if let Some(id) = name.strip_prefix("font-family-") {
            families.push(TokenRow {
                token,
                value,
                source: TokenSource::Css,
            });
            let _ = id;
            continue;
        }

        if let Some(id) = name.strip_prefix("font-size-") {
            font_sizes.insert(id.to_string(), value);
            continue;
        }

        if let Some(id) = name.strip_prefix("line-height-") {
            line_heights.insert(id.to_string(), value);
            continue;
        }

        if let Some(id) = name.strip_prefix("type-") {
            let id = id.strip_suffix("-size").unwrap_or(id).to_string();
            roles.push(TokenRow {
                token,
                value: raw_value.clone(),
                source: TokenSource::Css,
            });
            if let Some((font_size, line_height)) = split_type_pair(&value) {
                let (group, label) = style_names(&id, None, None);
                upsert_style(
                    &mut styles,
                    TypeStyle {
                        font_style: italic_from_id(&id),
                        id,
                        label,
                        group,
                        font_size,
                        line_height,
                        font_family: None,
                        source: TokenSource::Css,
                    },
                );
            }
            continue;
        }
    }

    for (id, font_size) in font_sizes {
        let line_height = line_heights.get(&id).cloned().unwrap_or_else(|| "1.2".into());
        if !styles.iter().any(|style| style.id == id) {
            let (group, label) = style_names(&id, None, None);
            styles.push(TypeStyle {
                id,
                label,
                group,
                font_size,
                line_height,
                font_family: None,
                font_style: None,
                source: TokenSource::Css,
            });
        }
    }

    let mut set = TokenSet::default();
    for mut scale in colors {
        sort_steps(&mut scale.steps);
        set.colors.push(scale);
    }
    set.typography.styles = styles;
    set.typography.families = families;
    set.typography.roles = roles;
    Ok(set)
}

fn color_parts(name: &str) -> Option<(String, String)> {
    let rest = name.strip_prefix("color-")?;
    let (family, step) = rest.rsplit_once('-')?;
    if family.is_empty() || parse_step_name(step).is_none() {
        return None;
    }
    Some((family.to_string(), step.to_string()))
}

fn parse_step_name(step: &str) -> Option<i32> {
    step.parse().ok()
}

fn italic_from_id(id: &str) -> Option<String> {
    if id.contains("italic") {
        Some("italic".into())
    } else {
        None
    }
}

fn upsert_style(styles: &mut Vec<TypeStyle>, incoming: TypeStyle) {
    if let Some(existing) = styles.iter_mut().find(|style| style.id == incoming.id) {
        *existing = incoming;
    } else {
        styles.push(incoming);
    }
}

fn split_type_pair(value: &str) -> Option<(String, String)> {
    let (size, height) = value.split_once(" / ")?;
    let size = size.trim();
    let height = height.trim();
    if size.is_empty() || height.is_empty() {
        return None;
    }
    Some((size.to_string(), height.to_string()))
}

fn collect_custom_properties(source: &str) -> Vec<(String, String)> {
    let stripped = strip_comments(source);
    let bytes = stripped.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
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
        if ch == b'"' || ch == b'\'' {
            quote = Some(ch);
            i += 1;
            continue;
        }
        if ch == b'-' && bytes.get(i + 1) == Some(&b'-') {
            let start = i + 2;
            let mut end = start;
            while end < bytes.len() {
                let next = bytes[end];
                if next.is_ascii_alphanumeric() || next == b'-' || next == b'_' {
                    end += 1;
                } else {
                    break;
                }
            }
            let mut cursor = end;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if bytes.get(cursor) == Some(&b':') {
                cursor += 1;
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if let Some((value, next)) = read_declaration_value(&stripped, cursor) {
                    let name = stripped[start..end].to_string();
                    if !name.is_empty() {
                        out.push((name, value));
                    }
                    i = next;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

fn read_declaration_value(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    let mut i = start;
    let mut quote: Option<u8> = None;
    let mut depth = 0i32;
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
            b'"' | b'\'' => quote = Some(ch),
            b'(' => depth += 1,
            b')' => depth -= 1,
            b';' if depth == 0 => {
                return Some((source[start..i].trim().to_string(), i + 1));
            }
            b'}' if depth == 0 => {
                return Some((source[start..i].trim().to_string(), i));
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let ch = bytes[i];
        if let Some(q) = quote {
            out.push(ch as char);
            if ch == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1] as char);
                i += 2;
                continue;
            }
            if ch == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            quote = Some(ch);
            out.push(ch as char);
            i += 1;
            continue;
        }
        if ch == b'/' && bytes.get(i + 1) == Some(&b'*') {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        out.push(ch as char);
        i += 1;
    }
    out
}

fn resolve_all(decls: &[(String, String)]) -> BTreeMap<String, String> {
    let raw: BTreeMap<String, String> = decls.iter().cloned().collect();
    let mut resolved = BTreeMap::new();
    for (name, value) in &raw {
        resolved.insert(name.clone(), resolve_vars(value, &raw, 0));
    }
    resolved
}

fn resolve_vars(value: &str, raw: &BTreeMap<String, String>, depth: u8) -> String {
    if depth > 8 {
        return value.to_string();
    }
    let mut out = String::new();
    let mut rest = value;
    while let Some(start) = rest.find("var(") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 4..];
        let end = match matching_paren(after) {
            Some(end) => end,
            None => {
                out.push_str(&rest[start..]);
                return out;
            }
        };
        let inner = after[..end].trim();
        let (name, fallback) = split_var_args(inner);
        let name = name.strip_prefix("--").unwrap_or(name);
        let replacement = raw
            .get(name)
            .map(|next| resolve_vars(next, raw, depth + 1))
            .or_else(|| fallback.map(str::to_string))
            .unwrap_or_else(|| format!("var({inner})"));
        out.push_str(&replacement);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

fn matching_paren(source: &str) -> Option<usize> {
    let mut depth = 1;
    for (index, ch) in source.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_var_args(inner: &str) -> (&str, Option<&str>) {
    match inner.split_once(',') {
        Some((name, fallback)) => (name.trim(), Some(fallback.trim())),
        None => (inner.trim(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
/* workshop tokens — adapter 4 */
:root {
  --color-yellow-50: #fffbeb;
  --color-yellow-500: #f59e0b;
  --color-yellow-950: #451a03;
  --font-size-display-xl: 4.5rem;
  --line-height-display-xl: 1.05;
  --type-display-xl: var(--font-size-display-xl) / var(--line-height-display-xl);
  --font-family-sans: "Inter Variable", sans-serif;
  --ignored: 1px; /* not a token we classify */
}
"#;

    #[test]
    fn walks_custom_properties_into_scales_and_type() {
        let set = parse_css_tokens(SAMPLE).unwrap();
        assert_eq!(set.colors.len(), 1);
        assert_eq!(set.colors[0].id, "yellow");
        assert_eq!(set.colors[0].name, "Yellow");
        assert_eq!(set.colors[0].source, TokenSource::Css);
        assert_eq!(set.colors[0].steps.len(), 3);
        assert_eq!(set.colors[0].steps[0].step, "50");
        assert_eq!(set.colors[0].steps[0].token, "--color-yellow-50");
        assert_eq!(set.typography.styles.len(), 1);
        assert_eq!(set.typography.styles[0].id, "display-xl");
        assert_eq!(set.typography.styles[0].font_size, "4.5rem");
        assert_eq!(set.typography.styles[0].line_height, "1.05");
        assert_eq!(set.typography.families[0].value, "\"Inter Variable\", sans-serif");
        assert_eq!(
            set.typography.roles[0].value,
            "var(--font-size-display-xl) / var(--line-height-display-xl)"
        );
    }

    #[test]
    fn skips_comments_and_quoted_semicolons() {
        let set = parse_css_tokens(
            r##"
:root {
  /* --color-ghost-50: #fff; */
  --color-pink-100: "#ff;pink";
}
"##,
        )
        .unwrap();
        assert_eq!(set.colors.len(), 1);
        assert_eq!(set.colors[0].id, "pink");
        assert_eq!(set.colors[0].steps[0].value, "\"#ff;pink\"");
    }
}
