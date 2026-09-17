//! Adapter 4: Tailwind theme variables from a CSS file.
//!
//! Tailwind-only for this MVP. Declarations are collected, then **only**
//! known Tailwind prefixes are classified. Unknown `--foo` names are ignored
//! — this is not a generic design-token map.
//!
//! Works on `@theme { … }` source and on compiled theme output (`:root`).
//!
//! # Prefix map
//!
//! | Prefix | Family | Notes |
//! | --- | --- | --- |
//! | `--color-*` | colors | `{palette}-{step}` when the last segment is numeric; else one swatch |
//! | `--text-*` | typography size, text color, or skip | see special cases below |
//! | `--text-*--line-height` | pairs with `--text-*` size | Tailwind size modifier, not a style of its own |
//! | `--text-shadow`, `--text-shadow-*` | text-shadow | not a font size |
//! | `--font-*` | font family | `--font-sans`, `--font-serif`, … |
//! | `--font-weight-*` | font-weight | not a family |
//! | `--leading-*` | line-height | |
//! | `--tracking-*` | letter-spacing | |
//! | `--spacing`, `--spacing-*` | spacing | |
//! | `--radius`, `--radius-*` | radius | |
//! | `--shadow-*` | box-shadow | |
//! | `--inset-shadow-*` | inset-shadow | |
//! | `--drop-shadow-*` | drop-shadow | |
//! | `--blur-*` | blur | |
//! | `--perspective-*` | perspective | |
//! | `--aspect-*` | aspect-ratio | |
//! | `--ease-*` | easing | |
//! | `--animate-*` | animation | |
//! | `--breakpoint-*` | breakpoints | |
//! | `--container-*` | containers | |
//!
//! `--text-*` special cases (size vs color vs other utilities):
//!
//! * `--text-{id}--line-height` → line-height companion for a size
//! * `--text-shadow*` → text-shadow family (above)
//! * alignment / wrap utilities (`left`, `center`, `right`, `justify`,
//!   `start`, `end`, `balance`, `pretty`, `wrap`, `nowrap`, `ellipsis`,
//!   `clip`) → ignored
//! * value looks like a color (`#`, `oklch(`, `var(--color-…)` …) → color
//!   scale `text`
//! * value looks like a length (`rem`, `px`, `calc(` …) → font size
//! * anything else → ignored
//!
//! There is no `--font-size-*` prefix in Tailwind. Font size is `--text-*`.

use std::collections::BTreeMap;

use crate::tokens::{
    sort_steps, style_names, ColorScale, ColorStep, TokenGroup, TokenRow, TokenSet, TokenSource,
    TypeStyle,
};

const TEXT_UTILITY_SKIP: &[&str] = &[
    "left", "center", "right", "justify", "start", "end", "balance", "pretty", "wrap", "nowrap",
    "ellipsis", "clip",
];

pub fn load_css_file(path: &std::path::Path) -> Result<TokenSet, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    parse_tailwind_tokens(&raw)
}

pub fn parse_css_tokens(source: &str) -> Result<TokenSet, String> {
    parse_tailwind_tokens(source)
}

pub fn parse_tailwind_tokens(source: &str) -> Result<TokenSet, String> {
    let decls = collect_custom_properties(source);
    let resolved: BTreeMap<String, String> = resolve_all(&decls);

    let mut colors: Vec<ColorScale> = Vec::new();
    let mut sizes: BTreeMap<String, String> = BTreeMap::new();
    let mut line_heights: BTreeMap<String, String> = BTreeMap::new();
    let mut styles: Vec<TypeStyle> = Vec::new();
    let mut families = Vec::new();
    let mut roles = Vec::new();
    let mut groups: Vec<TokenGroup> = Vec::new();

    for (name, raw_value) in &decls {
        let value = resolved.get(name).unwrap_or(raw_value).clone();
        match classify(name, &value) {
            Kind::Skip => {}
            Kind::Color { family, step } => {
                push_color(&mut colors, &family, &step, &value);
            }
            Kind::TextSize { id } => {
                sizes.insert(id, value);
            }
            Kind::TextLineHeight { id } => {
                line_heights.insert(id, value);
            }
            Kind::FontFamily { id } => {
                families.push(TokenRow {
                    token: format!("--font-{id}"),
                    value,
                    source: TokenSource::Css,
                });
            }
            Kind::Group { id, token } => {
                push_group_row(&mut groups, id, token, value);
            }
        }
    }

    for (id, font_size) in sizes {
        let line_height = line_heights.get(&id).cloned().unwrap_or_default();
        let token = format!("--text-{id}");
        let display = if line_height.is_empty() {
            font_size.clone()
        } else {
            format!("{font_size} / {line_height}")
        };
        roles.push(TokenRow {
            token,
            value: display,
            source: TokenSource::Css,
        });
        if !styles.iter().any(|style| style.id == id) {
            let (group, label) = style_names(&id, None, None);
            styles.push(TypeStyle {
                font_style: italic_from_id(&id),
                id,
                label,
                group,
                font_size,
                line_height,
                font_family: None,
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
    set.groups = groups;
    Ok(set)
}

enum Kind {
    Color { family: String, step: String },
    TextSize { id: String },
    TextLineHeight { id: String },
    FontFamily { id: String },
    Group { id: &'static str, token: String },
    Skip,
}

fn classify(name: &str, value: &str) -> Kind {
    if let Some(rest) = strip_exact_or_dash(name, "text-shadow") {
        return Kind::Group {
            id: "text-shadow",
            token: format!("--text-shadow{}", suffix_token(rest)),
        };
    }
    if let Some(rest) = strip_exact_or_dash(name, "inset-shadow") {
        return Kind::Group {
            id: "inset-shadow",
            token: format!("--inset-shadow{}", suffix_token(rest)),
        };
    }
    if let Some(rest) = strip_exact_or_dash(name, "drop-shadow") {
        return Kind::Group {
            id: "drop-shadow",
            token: format!("--drop-shadow{}", suffix_token(rest)),
        };
    }
    if let Some(rest) = name.strip_prefix("font-weight-") {
        return Kind::Group {
            id: "font-weight",
            token: format!("--font-weight-{rest}"),
        };
    }
    if let Some(rest) = name.strip_prefix("text-") {
        return classify_text(rest, value);
    }
    if let Some(rest) = name.strip_prefix("color-") {
        return classify_color(rest);
    }
    if let Some(rest) = name.strip_prefix("font-") {
        if rest.is_empty() {
            return Kind::Skip;
        }
        return Kind::FontFamily {
            id: rest.to_string(),
        };
    }
    for (prefix, family) in GROUP_PREFIXES {
        if let Some(rest) = strip_exact_or_dash(name, prefix) {
            return Kind::Group {
                id: family,
                token: format!("--{prefix}{}", suffix_token(rest)),
            };
        }
    }
    Kind::Skip
}

/// Tailwind families that are a flat token table (not colors or type styles).
const GROUP_PREFIXES: &[(&str, &str)] = &[
    ("leading", "leading"),
    ("tracking", "tracking"),
    ("spacing", "spacing"),
    ("radius", "radius"),
    ("shadow", "shadow"),
    ("blur", "blur"),
    ("perspective", "perspective"),
    ("aspect", "aspect"),
    ("ease", "ease"),
    ("animate", "animate"),
    ("breakpoint", "breakpoint"),
    ("container", "container"),
];

fn classify_text(rest: &str, value: &str) -> Kind {
    if let Some(id) = rest.strip_suffix("--line-height") {
        if id.is_empty() {
            return Kind::Skip;
        }
        return Kind::TextLineHeight { id: id.to_string() };
    }
    if TEXT_UTILITY_SKIP.contains(&rest) {
        return Kind::Skip;
    }
    if looks_like_color(value) {
        return Kind::Color {
            family: "text".into(),
            step: rest.to_string(),
        };
    }
    if looks_like_size(value) {
        return Kind::TextSize {
            id: rest.to_string(),
        };
    }
    Kind::Skip
}

fn classify_color(rest: &str) -> Kind {
    if rest.is_empty() {
        return Kind::Skip;
    }
    if let Some((family, step)) = rest.rsplit_once('-') {
        if !family.is_empty() && step.parse::<i32>().is_ok() {
            return Kind::Color {
                family: family.to_string(),
                step: step.to_string(),
            };
        }
    }
    Kind::Color {
        family: rest.to_string(),
        step: "DEFAULT".into(),
    }
}

fn looks_like_color(value: &str) -> bool {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();
    lower.starts_with('#')
        || lower.starts_with("rgb(")
        || lower.starts_with("rgba(")
        || lower.starts_with("hsl(")
        || lower.starts_with("hsla(")
        || lower.starts_with("oklch(")
        || lower.starts_with("oklab(")
        || lower.starts_with("lab(")
        || lower.starts_with("lch(")
        || lower.starts_with("hwb(")
        || lower.starts_with("color(")
        || lower.starts_with("color-mix(")
        || lower.starts_with("var(--color-")
}

fn looks_like_size(value: &str) -> bool {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();
    if lower.starts_with("calc(")
        || lower.starts_with("clamp(")
        || lower.starts_with("min(")
        || lower.starts_with("max(")
    {
        return true;
    }
    if lower.starts_with("var(--text-") || lower.starts_with("var(--spacing") {
        return true;
    }
    const UNITS: &[&str] = &[
        "rem", "em", "px", "pt", "pc", "cm", "mm", "in", "%", "ch", "ex", "ic", "cap", "lh", "rlh",
        "vw", "vh", "vmin", "vmax", "svw", "svh", "lvw", "lvh", "dvw", "dvh",
    ];
    UNITS.iter().any(|unit| {
        lower
            .strip_suffix(unit)
            .is_some_and(|head| head.ends_with(|ch: char| ch.is_ascii_digit() || ch == '.'))
    })
}

fn strip_exact_or_dash<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
    if name == prefix {
        return Some("");
    }
    name.strip_prefix(prefix)?.strip_prefix('-')
}

fn suffix_token(rest: &str) -> String {
    if rest.is_empty() {
        String::new()
    } else {
        format!("-{rest}")
    }
}

fn italic_from_id(id: &str) -> Option<String> {
    if id.contains("italic") {
        Some("italic".into())
    } else {
        None
    }
}

fn push_color(colors: &mut Vec<ColorScale>, family: &str, step: &str, value: &str) {
    let token = if step == "DEFAULT" {
        format!("--color-{family}")
    } else if family == "text" {
        format!("--text-{step}")
    } else {
        format!("--color-{family}-{step}")
    };
    if let Some(scale) = colors.iter_mut().find(|scale| scale.id == family) {
        scale.steps.push(ColorStep {
            step: step.to_string(),
            value: value.to_string(),
            token,
        });
        return;
    }
    colors.push(ColorScale {
        id: family.to_string(),
        name: crate::tokens::humanize(family),
        steps: vec![ColorStep {
            step: step.to_string(),
            value: value.to_string(),
            token,
        }],
        source: TokenSource::Css,
    });
}

fn push_group_row(groups: &mut Vec<TokenGroup>, id: &'static str, token: String, value: String) {
    let row = TokenRow {
        token,
        value,
        source: TokenSource::Css,
    };
    if let Some(group) = groups.iter_mut().find(|group| group.id == id) {
        group.rows.push(row);
        return;
    }
    groups.push(TokenGroup {
        id: id.to_string(),
        name: crate::tokens::humanize(id),
        rows: vec![row],
    });
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
@theme {
  --color-yellow-50: #fffbeb;
  --color-yellow-500: #f59e0b;
  --color-white: #ffffff;
  --text-display-xl: 4.5rem;
  --text-display-xl--line-height: 1.05;
  --text-primary: oklch(0.2 0 0);
  --text-center: center;
  --text-shadow-sm: 0 1px 2px rgb(0 0 0 / 0.1);
  --font-sans: "Inter Variable", sans-serif;
  --font-weight-bold: 700;
  --leading-tight: 1.25;
  --tracking-wide: 0.025em;
  --spacing: 0.25rem;
  --spacing-4: 1rem;
  --radius-md: 0.375rem;
  --shadow-sm: 0 1px 2px rgb(0 0 0 / 0.05);
  --inset-shadow-sm: inset 0 1px 2px rgb(0 0 0 / 0.05);
  --drop-shadow-sm: 0 1px 1px rgb(0 0 0 / 0.05);
  --blur-sm: 8px;
  --perspective-dramatic: 100px;
  --aspect-video: 16 / 9;
  --ease-in: cubic-bezier(0.4, 0, 1, 1);
  --animate-spin: spin 1s linear infinite;
  --breakpoint-md: 48rem;
  --container-3xl: 48rem;
  --font-size-display-xl: 99rem;
  --type-display-xl: ignored;
  --ignored: 1px;
}
"#;

    #[test]
    fn maps_tailwind_prefixes_and_ignores_the_rest() {
        let set = parse_tailwind_tokens(SAMPLE).unwrap();
        assert_eq!(
            set.colors
                .iter()
                .map(|scale| scale.id.as_str())
                .collect::<Vec<_>>(),
            vec!["yellow", "white", "text"]
        );
        assert_eq!(set.colors[0].steps[0].token, "--color-yellow-50");
        assert_eq!(set.colors[1].steps[0].token, "--color-white");
        assert_eq!(set.colors[1].steps[0].step, "DEFAULT");
        assert_eq!(set.colors[2].steps[0].token, "--text-primary");
        assert_eq!(set.typography.styles.len(), 1);
        assert_eq!(set.typography.styles[0].id, "display-xl");
        assert_eq!(set.typography.styles[0].font_size, "4.5rem");
        assert_eq!(set.typography.styles[0].line_height, "1.05");
        assert_eq!(set.typography.roles[0].token, "--text-display-xl");
        assert_eq!(set.typography.families[0].token, "--font-sans");
        let group_ids: Vec<_> = set.groups.iter().map(|group| group.id.as_str()).collect();
        for id in [
            "leading",
            "tracking",
            "spacing",
            "radius",
            "shadow",
            "inset-shadow",
            "drop-shadow",
            "blur",
            "perspective",
            "aspect",
            "ease",
            "animate",
            "breakpoint",
            "container",
            "text-shadow",
            "font-weight",
        ] {
            assert!(group_ids.contains(&id), "missing {id} in {group_ids:?}");
        }
        let spacing = set
            .groups
            .iter()
            .find(|group| group.id == "spacing")
            .unwrap();
        assert!(spacing.rows.iter().any(|row| row.token == "--spacing"));
        assert!(spacing.rows.iter().any(|row| row.token == "--spacing-4"));
        assert!(!set
            .typography
            .styles
            .iter()
            .any(|style| style.font_size == "99rem"));
        assert!(!set
            .groups
            .iter()
            .any(|group| group.rows.iter().any(|row| row.token == "--ignored")));
    }

    #[test]
    fn skips_comments_and_quoted_semicolons() {
        let set = parse_css_tokens(
            r##"
@theme {
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

    #[test]
    fn example_theme_covers_tailwind_families() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/tailwind-tokens/theme.css");
        let set = load_css_file(&path).unwrap();
        assert!(set.colors.iter().any(|scale| scale.id == "yellow"));
        assert!(set.colors.iter().any(|scale| scale.id == "white"));
        assert!(set.colors.iter().any(|scale| scale.id == "text"));
        assert!(set
            .typography
            .styles
            .iter()
            .any(|style| style.id == "display-xl"));
        assert!(set
            .typography
            .families
            .iter()
            .any(|row| row.token == "--font-sans"));
        assert!(!set
            .typography
            .styles
            .iter()
            .any(|style| style.font_size == "99rem"));
        let ids: Vec<_> = set.groups.iter().map(|group| group.id.as_str()).collect();
        for id in [
            "spacing",
            "radius",
            "shadow",
            "inset-shadow",
            "drop-shadow",
            "text-shadow",
            "font-weight",
            "leading",
            "tracking",
            "blur",
            "perspective",
            "aspect",
            "ease",
            "animate",
            "breakpoint",
            "container",
        ] {
            assert!(ids.contains(&id), "missing {id} in {ids:?}");
        }
    }
}
